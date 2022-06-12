use crate::chain::broadcaster::SenseiBroadcaster;
use crate::chain::manager::SenseiChainManager;
use crate::error::Error;
use crate::{chain::database::WalletDatabase, events::SenseiEvent, node::ChannelManager};
use bdk::{FeeRate, SignOptions};
use bitcoin::secp256k1::PublicKey;
use lightning::chain::chaininterface::ConfirmationTarget;
use lightning::util::config::{ChannelConfig, ChannelHandshakeLimits, UserConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

pub struct EventFilter<F>
where
    F: Fn(SenseiEvent) -> bool,
{
    pub f: F,
}

pub struct ChannelOpenRequest {
    pub node_connection_string: String,
    pub peer_pubkey: PublicKey,
    pub channel_amt_sat: u64,
    pub push_amt_msat: u64,
    pub custom_id: u64,
    pub announced_channel: bool,
}

pub struct ChannelOpener {
    node_id: String,
    channel_manager: Arc<ChannelManager>,
    wallet: Arc<Mutex<bdk::Wallet<WalletDatabase>>>,
    chain_manager: Arc<SenseiChainManager>,
    event_receiver: broadcast::Receiver<SenseiEvent>,
    broadcaster: Arc<SenseiBroadcaster>,
}

impl ChannelOpener {
    pub fn new(
        node_id: String,
        channel_manager: Arc<ChannelManager>,
        chain_manager: Arc<SenseiChainManager>,
        wallet: Arc<Mutex<bdk::Wallet<WalletDatabase>>>,
        event_receiver: broadcast::Receiver<SenseiEvent>,
        broadcaster: Arc<SenseiBroadcaster>,
    ) -> Self {
        Self {
            node_id,
            channel_manager,
            chain_manager,
            wallet,
            event_receiver,
            broadcaster,
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

    pub async fn open_batch(
        &mut self,
        requests: Vec<ChannelOpenRequest>,
    ) -> Vec<(ChannelOpenRequest, Result<[u8; 32], Error>)> {
        let mut requests_with_results = requests
            .into_iter()
            .map(|request| {
                let result = self.initiate_channel_open(&request);

                (request, result)
            })
            .collect::<Vec<_>>();

        let filters = requests_with_results
            .iter()
            .filter(|(_request, result)| result.is_ok())
            .map(|(request, _result)| {
                let filter_node_id = self.node_id.clone();
                let request_user_channel_id = request.custom_id;
                let filter = move |event| {
                    if let SenseiEvent::FundingGenerationReady {
                        node_id,
                        user_channel_id,
                        ..
                    } = event
                    {
                        if *node_id == filter_node_id && user_channel_id == request_user_channel_id
                        {
                            return true;
                        }
                    }
                    false
                };
                EventFilter { f: filter }
            })
            .collect::<Vec<EventFilter<_>>>();

        // TODO: is this appropriate timeout? maybe should accept as param
        let events = self.wait_for_events(filters, 30000, 500).await;

        // set error state for requests we didn't get an event for
        let requests_with_results = requests_with_results
            .drain(..)
            .map(|(request, result)| {
                if result.is_ok() {
                    let mut channel_counterparty_node_id = None;
                    let event = events.iter().find(|event| {
                        if let SenseiEvent::FundingGenerationReady {
                            user_channel_id,
                            counterparty_node_id,
                            ..
                        } = event
                        {
                            if *user_channel_id == request.custom_id {
                                channel_counterparty_node_id = Some(*counterparty_node_id);
                                return true;
                            }
                        }
                        false
                    });

                    if event.is_none() {
                        (request, Err(Error::FundingGenerationNeverHappened), None)
                    } else {
                        (request, result, channel_counterparty_node_id)
                    }
                } else {
                    (request, result, None)
                }
            })
            .collect::<Vec<_>>();

        // build a tx with these events and requests
        let wallet = self.wallet.lock().unwrap();

        let mut tx_builder = wallet.build_tx();
        let fee_sats_per_1000_wu = self
            .chain_manager
            .fee_estimator
            .get_est_sat_per_1000_weight(ConfirmationTarget::Normal);

        let sat_per_vb = std::cmp::min(1.0, fee_sats_per_1000_wu as f32 / 250.0);

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
        let (mut psbt, _tx_details) = tx_builder.finish().unwrap();
        let _finalized = wallet.sign(&mut psbt, SignOptions::default()).unwrap();
        let funding_tx = psbt.extract_tx();

        let channels_to_open = requests_with_results
            .iter()
            .filter(|(_request, result, _counterparty_node_id)| result.is_ok())
            .count();

        self.broadcaster
            .set_debounce(funding_tx.txid(), channels_to_open);

        requests_with_results
            .into_iter()
            .map(|(request, result, counterparty_node_id)| {
                if let Ok(tcid) = result {
                    let counterparty_node_id = counterparty_node_id.unwrap();
                    match self.channel_manager.funding_transaction_generated(
                        &tcid,
                        &counterparty_node_id,
                        funding_tx.clone(),
                    ) {
                        Ok(()) => (request, result),
                        Err(e) => (request, Err(Error::LdkApi(e))),
                    }
                } else {
                    (request, result)
                }
            })
            .collect()
    }

    fn initiate_channel_open(&self, request: &ChannelOpenRequest) -> Result<[u8; 32], Error> {
        let config = UserConfig {
            peer_channel_config_limits: ChannelHandshakeLimits {
                // lnd's max to_self_delay is 2016, so we want to be compatible.
                their_to_self_delay: 2016,
                ..Default::default()
            },
            channel_options: ChannelConfig {
                announced_channel: request.announced_channel,
                ..Default::default()
            },
            ..Default::default()
        };

        // TODO: want to be logging channels in db for matching forwarded payments
        match self.channel_manager.create_channel(
            request.peer_pubkey,
            request.channel_amt_sat,
            request.push_amt_msat,
            request.custom_id,
            Some(config),
        ) {
            Ok(short_channel_id) => {
                println!(
                    "EVENT: initiated channel with peer {}. ",
                    request.peer_pubkey
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
