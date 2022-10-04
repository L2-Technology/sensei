// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::chain::broadcaster::SenseiBroadcaster;
use crate::chain::database::WalletDatabase;
use crate::chain::manager::SenseiChainManager;
use crate::config::SenseiConfig;
use crate::database::SenseiDatabase;
use crate::events::SenseiEvent;
use crate::hex_utils;
use crate::node::{ChannelManager, HTLCStatus, PaymentOrigin};

use bdk::wallet::AddressIndex;
use bitcoin::secp256k1::{self, PublicKey};
use bitcoin::{secp256k1::Secp256k1, Network};
use bitcoin_bech32::WitnessProgram;
use entity::sea_orm::ActiveValue;
use lightning::chain::chaininterface::{BroadcasterInterface, FeeEstimator};
use lightning::chain::keysinterface::{KeysInterface, PhantomKeysManager, Recipient};

use lightning::{
    chain::chaininterface::ConfirmationTarget,
    util::events::{Event, EventHandler, PaymentPurpose},
};
use rand::{thread_rng, Rng};
use std::sync::Mutex;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Handle;
use tokio::sync::broadcast;

pub struct LightningNodeEventHandler {
    pub node_id: String,
    pub config: Arc<SenseiConfig>,
    pub wallet: Arc<Mutex<bdk::Wallet<WalletDatabase>>>,
    pub channel_manager: Arc<ChannelManager>,
    pub keys_manager: Arc<PhantomKeysManager>,
    pub database: Arc<SenseiDatabase>,
    pub chain_manager: Arc<SenseiChainManager>,
    pub tokio_handle: Handle,
    pub event_sender: broadcast::Sender<SenseiEvent>,
    pub broadcaster: Arc<SenseiBroadcaster>,
    pub secp_ctx: Secp256k1<secp256k1::All>,
}

impl EventHandler for LightningNodeEventHandler {
    fn handle_event(&self, event: &Event) {
        match event {
            Event::OpenChannelRequest {
                temporary_channel_id,
                counterparty_node_id,
                funding_satoshis: _,
                push_msat: _,
                channel_type: _,
            } => {
                let is_trusted_peer = match self
                    .database
                    .find_peer_sync(&self.node_id, &counterparty_node_id.to_string())
                {
                    Ok(Some(known_peer)) => known_peer.zero_conf,
                    _ => false,
                };

                if is_trusted_peer {
                    match self
                        .channel_manager
                        .accept_inbound_channel_from_trusted_peer_0conf(
                            temporary_channel_id,
                            counterparty_node_id,
                            1,
                        ) {
                        Ok(()) => {
                            println!(
                                "accepted 0-conf inbound channel from {:?}",
                                counterparty_node_id
                            );
                        }
                        Err(e) => {
                            println!(
                                "failed to accept 0-conf inbound channel from {:?}: {:?}",
                                counterparty_node_id, e
                            );
                        }
                    }
                } else {
                    match self.channel_manager.accept_inbound_channel(
                        temporary_channel_id,
                        counterparty_node_id,
                        1,
                    ) {
                        Ok(()) => {
                            println!("accepted inbound channel from {:?}", counterparty_node_id);
                        }
                        Err(e) => {
                            println!(
                                "failed to accept inbound channel from {:?}: {:?}",
                                counterparty_node_id, e
                            );
                        }
                    }
                }
            }
            Event::FundingGenerationReady {
                temporary_channel_id,
                channel_value_satoshis,
                output_script,
                counterparty_node_id,
                user_channel_id,
            } => {
                // Construct the raw transaction with one output, that is paid the amount of the
                // channel.
                let _addr = WitnessProgram::from_scriptpubkey(
                    &output_script[..],
                    match self.config.network {
                        Network::Bitcoin => bitcoin_bech32::constants::Network::Bitcoin,
                        Network::Testnet => bitcoin_bech32::constants::Network::Testnet,
                        Network::Regtest => bitcoin_bech32::constants::Network::Regtest,
                        Network::Signet => bitcoin_bech32::constants::Network::Signet,
                    },
                )
                .expect("Lightning funding tx should always be to a SegWit output")
                .to_address();

                let _res = self
                    .event_sender
                    .send(SenseiEvent::FundingGenerationReady {
                        node_id: self.node_id.clone(),
                        temporary_channel_id: *temporary_channel_id,
                        channel_value_satoshis: *channel_value_satoshis,
                        output_script: output_script.clone(),
                        user_channel_id: *user_channel_id,
                        counterparty_node_id: *counterparty_node_id,
                    })
                    .unwrap();
            }
            Event::PaymentReceived {
                payment_hash,
                purpose,
                amount_msat,
                ..
            } => {
                println!(
                    "\nEVENT: received payment from payment hash {} of {} millisatoshis",
                    hex_utils::hex_str(&payment_hash.0),
                    amount_msat
                );

                let payment_preimage = match purpose {
                    PaymentPurpose::InvoicePayment {
                        payment_preimage, ..
                    } => *payment_preimage,
                    PaymentPurpose::SpontaneousPayment(preimage) => Some(*preimage),
                };

                // TODO: if we want 'hodl invoices' we should have user set a flag on the invoice when they create it
                //       then when we receive this event we can store the preimage + flag in db for this payment
                //       user can then manually accept it
                //        or maybe defines some custom logic on if/when to accept it
                self.channel_manager.claim_funds(payment_preimage.unwrap());
            }
            Event::PaymentClaimed {
                payment_hash,
                purpose,
                amount_msat,
            } => {
                println!(
                    "\nEVENT: claimed payment from payment hash {} of {} millisatoshis",
                    hex_utils::hex_str(&payment_hash.0),
                    amount_msat,
                );

                let (payment_preimage, payment_secret) = match purpose {
                    PaymentPurpose::InvoicePayment {
                        payment_preimage,
                        payment_secret,
                        ..
                    } => (*payment_preimage, Some(*payment_secret)),
                    PaymentPurpose::SpontaneousPayment(preimage) => (Some(*preimage), None),
                };

                let payment_hash = hex_utils::hex_str(&payment_hash.0);
                let preimage = payment_preimage.map(|preimage| hex_utils::hex_str(&preimage.0));
                let secret = payment_secret.map(|secret| hex_utils::hex_str(&secret.0));
                let amt_msat: Option<i64> = Some((*amount_msat).try_into().unwrap());

                let existing_payment = self
                    .database
                    .find_payment_sync(self.node_id.clone(), payment_hash.clone())
                    .unwrap_or_else(|_| {
                        let phantom_node_secret = self
                            .keys_manager
                            .get_node_secret(Recipient::PhantomNode)
                            .unwrap();
                        let phantom_node_pubkey =
                            PublicKey::from_secret_key(&self.secp_ctx, &phantom_node_secret)
                                .to_string();
                        self.database
                            .find_payment_sync(phantom_node_pubkey, payment_hash.clone())
                            .unwrap_or(None)
                    });

                match existing_payment {
                    Some(payment) => {
                        let mut payment: entity::payment::ActiveModel = payment.into();
                        payment.status = ActiveValue::Set(HTLCStatus::Succeeded.to_string());
                        payment.received_by_node_id = ActiveValue::Set(Some(self.node_id.clone()));
                        payment.preimage = ActiveValue::Set(preimage);
                        payment.secret = ActiveValue::Set(secret);
                        payment.amt_msat = ActiveValue::Set(amt_msat);

                        self.database.update_payment_sync(payment).unwrap();
                    }
                    None => {
                        let payment = entity::payment::ActiveModel {
                            node_id: ActiveValue::Set(self.node_id.clone()),
                            created_by_node_id: ActiveValue::Set(self.node_id.clone()),
                            received_by_node_id: ActiveValue::Set(Some(self.node_id.clone())),
                            payment_hash: ActiveValue::Set(payment_hash),
                            status: ActiveValue::Set(HTLCStatus::Succeeded.to_string()),
                            preimage: ActiveValue::Set(preimage),
                            secret: ActiveValue::Set(secret),
                            amt_msat: ActiveValue::Set(amt_msat),
                            origin: ActiveValue::Set(
                                PaymentOrigin::SpontaneousIncoming.to_string(),
                            ),
                            ..Default::default()
                        };

                        self.database.insert_payment_sync(payment).unwrap();
                    }
                };
            }
            Event::PaymentSent {
                payment_preimage,
                payment_hash,
                fee_paid_msat,
                ..
            } => {
                let hex_payment_hash = hex_utils::hex_str(&payment_hash.0);

                let payment = self
                    .database
                    .find_payment_sync(self.node_id.clone(), hex_payment_hash);

                if let Ok(Some(payment)) = payment {
                    let amt_msat = payment.amt_msat;

                    let mut payment: entity::payment::ActiveModel = payment.into();
                    payment.preimage =
                        ActiveValue::Set(Some(hex_utils::hex_str(&payment_preimage.0)));
                    payment.status = ActiveValue::Set(HTLCStatus::Succeeded.to_string());

                    let _res = self.database.update_payment_sync(payment);

                    println!(
                        "\nEVENT: successfully sent payment of {:?} millisatoshis{} from \
                                    payment hash {:?} with preimage {:?}",
                        amt_msat,
                        if let Some(fee) = fee_paid_msat {
                            format!(" (fee {} msat)", fee)
                        } else {
                            "".to_string()
                        },
                        hex_utils::hex_str(&payment_hash.0),
                        hex_utils::hex_str(&payment_preimage.0)
                    );
                }
            }
            Event::PaymentPathSuccessful { path: _, .. } => {}
            Event::PaymentPathFailed {
                path: _,
                short_channel_id: _,
                ..
            } => {}
            Event::PaymentFailed { payment_hash, .. } => {
                print!(
                    "\nEVENT: Failed to send payment to payment hash {:?}: exhausted payment retry attempts",
				    hex_utils::hex_str(&payment_hash.0)
                );

                let hex_payment_hash = hex_utils::hex_str(&payment_hash.0);

                let payment = self
                    .database
                    .find_payment_sync(self.node_id.clone(), hex_payment_hash);

                if let Ok(Some(payment)) = payment {
                    let mut payment: entity::payment::ActiveModel = payment.into();
                    payment.status = ActiveValue::Set(HTLCStatus::Failed.to_string());

                    let res = self.database.update_payment_sync(payment);

                    match res {
                        Ok(_payment) => {}
                        Err(_e) => {
                            println!("failed to update payment");
                        }
                    }
                }
            }
            Event::PaymentForwarded {
                prev_channel_id,
                next_channel_id,
                fee_earned_msat,
                claim_from_onchain_tx,
            } => {
                let channels = self.channel_manager.list_channels();

                let node_str = |channel_id: &Option<[u8; 32]>| match channel_id {
                    None => String::new(),
                    Some(channel_id) => match channels.iter().find(|c| c.channel_id == *channel_id)
                    {
                        None => String::new(),
                        Some(channel) => {
                            format!(
                                " from private node {:?}",
                                channel.counterparty.node_id.to_string()
                            )
                        }
                    },
                };
                let channel_str = |channel_id: &Option<[u8; 32]>| {
                    channel_id
                        .map(|channel_id| {
                            format!(" with channel {}", hex_utils::hex_str(&channel_id))
                        })
                        .unwrap_or_default()
                };
                let from_prev_str = format!(
                    "{}{}",
                    node_str(prev_channel_id),
                    channel_str(prev_channel_id)
                );
                let to_next_str = format!(
                    "{}{}",
                    node_str(next_channel_id),
                    channel_str(next_channel_id)
                );

                let from_onchain_str = if *claim_from_onchain_tx {
                    "from onchain downstream claim"
                } else {
                    "from HTLC fulfill message"
                };
                if let Some(fee_earned) = fee_earned_msat {
                    println!(
                        "\nEVENT: Forwarded payment{}{}, earning {} msat {}",
                        from_prev_str, to_next_str, fee_earned, from_onchain_str
                    );

                    // let forwarded_payment = ForwardedPayment {
                    //     hours_since_epoch: utils::hours_since_epoch().unwrap(),
                    //     from_channel_id: None,
                    //     to_channel_id: None,
                    //     fees_earned_msat: *fee_earned,
                    //     total_payments: 1,
                    // };

                    // let mut database = self.database.lock().unwrap();
                    // database
                    //     .record_forwarded_payment(forwarded_payment)
                    //     .unwrap();
                } else {
                    println!(
                        "\nEVENT: Forwarded payment{}{}, claiming onchain {}",
                        from_prev_str, to_next_str, from_onchain_str
                    );
                }
            }
            Event::PendingHTLCsForwardable { time_forwardable } => {
                let forwarding_channel_manager = self.channel_manager.clone();
                let min = time_forwardable.as_millis() as u64;
                self.tokio_handle.spawn(async move {
                    let millis_to_sleep = thread_rng().gen_range(min..(min * 5)) as u64;
                    tokio::time::sleep(Duration::from_millis(millis_to_sleep)).await;
                    forwarding_channel_manager.process_pending_htlc_forwards();
                });
            }
            Event::SpendableOutputs { outputs } => {
                let wallet = self.wallet.lock().unwrap();
                let address_info = wallet.get_address(AddressIndex::LastUnused).unwrap();
                let destination_address = address_info.address;
                let output_descriptors = &outputs.iter().collect::<Vec<_>>();

                let tx_feerate = self
                    .chain_manager
                    .fee_estimator
                    .get_est_sat_per_1000_weight(ConfirmationTarget::Normal);

                let spending_tx = self
                    .keys_manager
                    .spend_spendable_outputs(
                        output_descriptors,
                        Vec::new(),
                        destination_address.script_pubkey(),
                        tx_feerate,
                        &Secp256k1::new(),
                    )
                    .unwrap();

                self.broadcaster.broadcast_transaction(&spending_tx);
            }
            Event::ChannelClosed {
                channel_id,
                reason,
                user_channel_id,
            } => {
                println!(
                    "\nEVENT: Channel {} closed due to: {:?}",
                    hex_utils::hex_str(channel_id),
                    reason
                );

                let reason = match reason {
                    lightning::util::events::ClosureReason::CounterpartyForceClosed {
                        peer_msg,
                    } => peer_msg.clone(),
                    lightning::util::events::ClosureReason::ProcessingError { err } => err.clone(),
                    _ => format!("{:}", reason),
                };

                let _res = self
                    .event_sender
                    .send(SenseiEvent::ChannelClosed {
                        node_id: self.node_id.clone(),
                        channel_id: *channel_id,
                        user_channel_id: *user_channel_id,
                        reason,
                    })
                    .unwrap();
            }
            Event::DiscardFunding { .. } => {
                // A "real" node should probably "lock" the UTXOs spent in funding transactions until
                // the funding transaction either confirms, or this event is generated.
            }
            Event::ProbeSuccessful { .. } => {}
            Event::ProbeFailed { .. } => {}
            Event::HTLCHandlingFailed { .. } => {}
        }
    }
}
