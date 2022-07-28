use crate::chain::broadcaster::SenseiBroadcaster;
use crate::chain::manager::SenseiChainManager;
use crate::error::Error;
use crate::node::PeerManager;
use crate::p2p::peer_connector::PeerConnector;
use crate::p2p::utils::{parse_peer_addr, parse_pubkey};
use crate::services::node::OpenChannelRequest;
use crate::{chain::database::WalletDatabase, events::SenseiEvent, node::ChannelManager};
use bdk::{FeeRate, SignOptions};
use lightning::chain::chaininterface::{ConfirmationTarget, FeeEstimator};
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

pub struct EventFilter<F>
where
    F: Fn(SenseiEvent) -> bool,
{
    pub f: F,
}

pub struct ChannelOpener {
    node_id: String,
    channel_manager: Arc<ChannelManager>,
    wallet: Arc<Mutex<bdk::Wallet<WalletDatabase>>>,
    chain_manager: Arc<SenseiChainManager>,
    event_receiver: broadcast::Receiver<SenseiEvent>,
    broadcaster: Arc<SenseiBroadcaster>,
    peer_manager: Arc<PeerManager>,
    peer_connector: Arc<PeerConnector>,
}

impl ChannelOpener {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: String,
        channel_manager: Arc<ChannelManager>,
        chain_manager: Arc<SenseiChainManager>,
        wallet: Arc<Mutex<bdk::Wallet<WalletDatabase>>>,
        event_receiver: broadcast::Receiver<SenseiEvent>,
        broadcaster: Arc<SenseiBroadcaster>,
        peer_manager: Arc<PeerManager>,
        peer_connector: Arc<PeerConnector>,
    ) -> Self {
        Self {
            node_id,
            channel_manager,
            chain_manager,
            wallet,
            event_receiver,
            broadcaster,
            peer_manager,
            peer_connector,
        }
    }

    async fn wait_for_events<F: Fn(SenseiEvent) -> bool>(
        &mut self,
        mut filters: Vec<EventFilter<F>>,
        timeout_ms: u64,
        interval_ms: u64,
    ) -> Vec<SenseiEvent> {
        let mut events = vec![];
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            while let Ok(event) = self.event_receiver.try_recv() {
                let filter_index = filters
                    .iter()
                    .enumerate()
                    .find(|(_index, filter)| (filter.f)(event.clone()))
                    .map(|(index, _filter)| index);

                if let Some(index) = filter_index {
                    events.push(event);
                    filters.swap_remove(index);
                }

                if filters.is_empty() {
                    return events;
                }
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }
        events
    }

    fn ensure_custom_ids(&self, requests: Vec<OpenChannelRequest>) -> Vec<OpenChannelRequest> {
        requests
            .into_iter()
            .map(|request| OpenChannelRequest {
                custom_id: Some(
                    request
                        .custom_id
                        .unwrap_or_else(|| thread_rng().gen_range(1..u64::MAX)),
                ),
                ..request
            })
            .collect::<Vec<_>>()
    }

    fn event_filter_for_request(
        &self,
        request: &OpenChannelRequest,
    ) -> impl Fn(SenseiEvent) -> bool {
        let filter_node_id = self.node_id.clone();
        let request_user_channel_id = request.custom_id.unwrap();
        move |event| match event {
            SenseiEvent::FundingGenerationReady {
                node_id,
                user_channel_id,
                ..
            } => *node_id == filter_node_id && user_channel_id == request_user_channel_id,
            SenseiEvent::ChannelClosed {
                node_id,
                user_channel_id,
                ..
            } => *node_id == filter_node_id && user_channel_id == request_user_channel_id,
            _ => false,
        }
    }

    pub async fn open_batch(
        &mut self,
        requests: Vec<OpenChannelRequest>,
    ) -> Result<Vec<(OpenChannelRequest, Result<[u8; 32], Error>)>, Error> {
        let requests = self.ensure_custom_ids(requests);
        let mut requests_with_results = vec![];
        let mut filters = vec![];

        for request in requests {
            let result = self.initiate_channel_open(&request).await;
            if result.is_ok() {
                filters.push(EventFilter {
                    f: self.event_filter_for_request(&request),
                })
            }
            requests_with_results.push((request, result));
        }

        // TODO: is this appropriate timeout? maybe should accept as param
        let events = self.wait_for_events(filters, 30000, 500).await;

        // set error state for requests we didn't get an event for
        let requests_with_results = requests_with_results
            .drain(..)
            .map(|(request, result)| {
                if result.is_ok() {
                    let event_opt = events.iter().find(|event| match event {
                        SenseiEvent::FundingGenerationReady {
                            user_channel_id, ..
                        } => *user_channel_id == request.custom_id.unwrap(),
                        SenseiEvent::ChannelClosed {
                            user_channel_id, ..
                        } => *user_channel_id == request.custom_id.unwrap(),
                        _ => false,
                    });

                    match event_opt {
                        Some(SenseiEvent::FundingGenerationReady {
                            counterparty_node_id,
                            ..
                        }) => (request, result, Some(*counterparty_node_id)),
                        Some(SenseiEvent::ChannelClosed { reason, .. }) => (
                            request,
                            Err(Error::ChannelOpenRejected(reason.clone())),
                            None,
                        ),
                        _ => (request, Err(Error::FundingGenerationNeverHappened), None),
                    }
                } else {
                    (request, result, None)
                }
            })
            .collect::<Vec<_>>();

        let ok_results = requests_with_results
            .iter()
            .filter(|(_, result, _)| result.is_ok())
            .count();

        if ok_results == 0 {
            return Ok(requests_with_results
                .into_iter()
                .map(|(req, res, _)| (req, res))
                .collect::<Vec<_>>());
        }

        // build a tx with these events and requests
        let wallet = self.wallet.lock().unwrap();

        let mut tx_builder = wallet.build_tx();
        let fee_sats_per_1000_wu = self
            .chain_manager
            .fee_estimator
            .get_est_sat_per_1000_weight(ConfirmationTarget::Normal);

        // TODO: is this the correct conversion??
        let sat_per_vb = match fee_sats_per_1000_wu {
            253 => 1.0,
            _ => fee_sats_per_1000_wu as f32 / 250.0,
        } as f32;

        let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

        events.iter().for_each(|event| {
            if let SenseiEvent::FundingGenerationReady {
                channel_value_satoshis,
                output_script,
                ..
            } = event
            {
                tx_builder.add_recipient(output_script.clone(), *channel_value_satoshis);
            }
        });

        tx_builder.fee_rate(fee_rate).enable_rbf();
        let tx_result = tx_builder.finish();

        if let Err(e) = tx_result {
            for (_request, result, counterparty) in requests_with_results.iter() {
                if let Ok(tcid) = result {
                    let _res = self
                        .channel_manager
                        .force_close_broadcasting_latest_txn(tcid, counterparty.as_ref().unwrap());
                }
            }
            return Err(Error::Bdk(e));
        }

        let (mut psbt, _tx_details) = tx_result.unwrap();

        let _finalized = wallet.sign(&mut psbt, SignOptions::default()).unwrap();
        let funding_tx = psbt.extract_tx();

        let channels_to_open = requests_with_results
            .iter()
            .filter(|(_request, result, _counterparty_node_id)| result.is_ok())
            .count();

        self.broadcaster
            .set_debounce(funding_tx.txid(), channels_to_open);

        let requests_with_results = requests_with_results
            .into_iter()
            .map(|(request, result, counterparty_node_id)| {
                if let Ok(tcid) = result {
                    let counterparty_node_id = counterparty_node_id.unwrap();
                    match self.channel_manager.funding_transaction_generated(
                        &tcid,
                        &counterparty_node_id,
                        funding_tx.clone(),
                    ) {
                        Ok(()) => {
                            let channels = self.channel_manager.list_channels();
                            let channel = channels.iter().find(|channel| {
                                channel.user_channel_id == request.custom_id.unwrap()
                            });
                            let channel = channel.expect("to find channel we opened");
                            (request, Ok(channel.channel_id))
                        }
                        Err(e) => (request, Err(Error::LdkApi(e))),
                    }
                } else {
                    (request, result)
                }
            })
            .collect();

        Ok(requests_with_results)
    }

    async fn initiate_channel_open(&self, request: &OpenChannelRequest) -> Result<[u8; 32], Error> {
        let counterparty_pubkey =
            parse_pubkey(&request.counterparty_pubkey).expect("failed to parse pubkey");
        let already_connected = self
            .peer_manager
            .get_peer_node_ids()
            .contains(&counterparty_pubkey);
        if !already_connected {
            let counterparty_host_port = request.counterparty_host_port.as_ref().expect("you must provide connection information if you are not already connected to a peer");
            let counterparty_addr = parse_peer_addr(counterparty_host_port)
                .await
                .expect("failed to parse host port for counterparty");
            self.peer_connector
                .connect_peer_if_necessary(
                    &self.node_id,
                    counterparty_pubkey,
                    counterparty_addr.clone(),
                    self.peer_manager.clone(),
                )
                .await
                .unwrap_or_else(|_| {
                    panic!(
                        "failed to connect to peer {}@{:?}",
                        counterparty_pubkey, counterparty_addr
                    )
                });
        }

        // TODO: want to be logging channels in db for matching forwarded payments
        match self.channel_manager.create_channel(
            counterparty_pubkey,
            request.amount_sats,
            request.push_amount_msats.unwrap_or(0),
            request.custom_id.unwrap(),
            Some(request.into()),
        ) {
            Ok(short_channel_id) => {
                println!(
                    "EVENT: initiated channel with peer {}. ",
                    request.counterparty_pubkey
                );
                Ok(short_channel_id)
            }
            Err(e) => {
                println!("ERROR: failed to open channel: {:?}", e);
                Err(e.into())
            }
        }
    }
}
