// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::database::node::{ForwardedPayment, NodeDatabase};
use crate::node::{
    ChannelManager, HTLCStatus, LightningWallet, MillisatAmount, PaymentInfo, PaymentOrigin,
};
use crate::utils;
use crate::{config::LightningNodeConfig, hex_utils};

use bitcoin::{secp256k1::Secp256k1, Network};
use bitcoin_bech32::WitnessProgram;
use lightning::{
    chain::{
        chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator},
        keysinterface::KeysManager,
    },
    util::events::{Event, EventHandler, PaymentPurpose},
};
use rand::{thread_rng, Rng};
use std::sync::Mutex;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Handle;

pub struct LightningNodeEventHandler {
    pub config: LightningNodeConfig,
    pub wallet: Arc<LightningWallet>,
    pub channel_manager: Arc<ChannelManager>,
    pub keys_manager: Arc<KeysManager>,
    pub database: Arc<Mutex<NodeDatabase>>,
    pub tokio_handle: Handle,
}

impl EventHandler for LightningNodeEventHandler {
    fn handle_event(&self, event: &Event) {
        match event {
            Event::FundingGenerationReady {
                temporary_channel_id,
                channel_value_satoshis,
                output_script,
                user_channel_id: _,
            } => {
                // Construct the raw transaction with one output, that is paid the amount of the
                // channel.
                let _addr = WitnessProgram::from_scriptpubkey(
                    &output_script[..],
                    match self.config.network {
                        Network::Bitcoin => bitcoin_bech32::constants::Network::Bitcoin,
                        Network::Testnet => bitcoin_bech32::constants::Network::Testnet,
                        Network::Regtest => bitcoin_bech32::constants::Network::Regtest,
                        Network::Signet => panic!("Signet unsupported"),
                    },
                )
                .expect("Lightning funding tx should always be to a SegWit output")
                .to_address();

                let target_blocks = 100;

                // Have wallet put the inputs into the transaction such that the output
                // is satisfied and then sign the funding transaction
                let funding_tx = self
                    .wallet
                    .construct_funding_transaction(
                        output_script,
                        *channel_value_satoshis,
                        target_blocks,
                    )
                    .unwrap();

                // Give the funding transaction back to LDK for opening the channel.
                if self
                    .channel_manager
                    .funding_transaction_generated(temporary_channel_id, funding_tx)
                    .is_err()
                {
                    println!(
                        "\nERROR: Channel went away before we could fund it. The peer disconnected or refused the channel.");
                }
            }
            Event::PaymentReceived {
                payment_hash,
                purpose,
                amt,
                ..
            } => {
                // inbound...
                let mut database = self.database.lock().unwrap();

                let (payment_preimage, payment_secret) = match purpose {
                    PaymentPurpose::InvoicePayment {
                        payment_preimage,
                        payment_secret,
                        ..
                    } => (*payment_preimage, Some(*payment_secret)),
                    PaymentPurpose::SpontaneousPayment(preimage) => (Some(*preimage), None),
                };

                // TODO: if we want 'hodl invoices' we should have user set a flag on the invoice when they create it
                //       then when we receive this event we can store the preimage + flag in db for this payment
                //       user can then manually accept it
                //        or maybe defines some custom logic on if/when to accept it
                let status = match self.channel_manager.claim_funds(payment_preimage.unwrap()) {
                    true => {
                        println!(
                            "\nEVENT: received payment from payment hash {} of {} millisatoshis",
                            hex_utils::hex_str(&payment_hash.0),
                            amt
                        );

                        HTLCStatus::Succeeded
                    }
                    _ => HTLCStatus::Failed,
                };

                // TODO: handle error reading here in a better way?
                let existing_payment = database
                    .get_payment(hex_utils::hex_str(&payment_hash.0))
                    .unwrap_or(None);

                let (label, invoice) = match existing_payment {
                    Some(payment) => (payment.label, payment.invoice),
                    None => (None, None),
                };

                let payment = PaymentInfo {
                    hash: *payment_hash,
                    preimage: payment_preimage,
                    secret: payment_secret,
                    status,
                    amt_msat: MillisatAmount(Some(*amt)),
                    origin: PaymentOrigin::InvoiceIncoming,
                    label,
                    invoice,
                };

                // TODO: in this case we probably already set the label so we shouldn't be
                // overriding it -- maybe need to find first and then update.
                let res = database.create_or_update_payment(payment.into());

                match res {
                    Ok(()) => {}
                    Err(_e) => {
                        println!("failed to update payment");
                    }
                }
            }
            Event::PaymentSent {
                payment_preimage,
                payment_hash,
                fee_paid_msat,
                ..
            } => {
                // outbound
                let mut database = self.database.lock().unwrap();
                let payment = database.get_payment(hex_utils::hex_str(&payment_hash.0));

                if let Ok(Some(mut payment)) = payment {
                    // update payment
                    payment.preimage = Some(hex_utils::hex_str(&payment_preimage.0));
                    payment.status = HTLCStatus::Succeeded.to_string();

                    let amt_msat = payment.amt_msat;
                    let _res = database.create_or_update_payment(payment);

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
            Event::PaymentPathSuccessful { .. } => {}
            Event::PaymentPathFailed { .. } => {}
            Event::PaymentFailed { payment_hash, .. } => {
                print!(
                    "\nEVENT: Failed to send payment to payment hash {:?}: exhausted payment retry attempts",
				    hex_utils::hex_str(&payment_hash.0)
                );

                let mut database = self.database.lock().unwrap();
                let payment = database.get_payment(hex_utils::hex_str(&payment_hash.0));

                if let Ok(Some(mut payment)) = payment {
                    // update payment
                    payment.status = HTLCStatus::Failed.to_string();
                    let res = database.create_or_update_payment(payment);
                    match res {
                        Ok(()) => {}
                        Err(_e) => {
                            println!("failed to update payment");
                        }
                    }
                }
            }
            Event::PaymentForwarded {
                fee_earned_msat,
                claim_from_onchain_tx,
            } => {
                let from_onchain_str = if *claim_from_onchain_tx {
                    "from onchain downstream claim"
                } else {
                    "from HTLC fulfill message"
                };
                if let Some(fee_earned) = fee_earned_msat {
                    println!(
                        "\nEVENT: Forwarded payment, earning {} msat {}",
                        fee_earned, from_onchain_str
                    );

                    let forwarded_payment = ForwardedPayment {
                        hours_since_epoch: utils::hours_since_epoch().unwrap(),
                        from_channel_id: None,
                        to_channel_id: None,
                        fees_earned_msat: *fee_earned,
                        total_payments: 1,
                    };

                    let mut database = self.database.lock().unwrap();
                    database
                        .record_forwarded_payment(forwarded_payment)
                        .unwrap();
                } else {
                    println!(
                        "\nEVENT: Forwarded payment, claiming onchain {}",
                        from_onchain_str
                    );
                }
            }
            Event::PendingHTLCsForwardable { time_forwardable } => {
                let forwarding_channel_manager = self.channel_manager.clone();
                let min = time_forwardable.as_millis() as u64;
                self.tokio_handle.spawn(async move {
                    let millis_to_sleep = thread_rng().gen_range(min, min * 5) as u64;
                    tokio::time::sleep(Duration::from_millis(millis_to_sleep)).await;
                    forwarding_channel_manager.process_pending_htlc_forwards();
                });
            }
            Event::SpendableOutputs { outputs } => {
                let destination_address = self.wallet.get_unused_address().unwrap();
                let output_descriptors = &outputs.iter().collect::<Vec<_>>();
                let tx_feerate = self
                    .wallet
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
                self.wallet.broadcast_transaction(&spending_tx);
            }
            Event::ChannelClosed {
                channel_id,
                reason,
                user_channel_id: _,
            } => {
                println!(
                    "\nEVENT: Channel {} closed due to: {:?}",
                    hex_utils::hex_str(channel_id),
                    reason
                );
            }
            Event::DiscardFunding { .. } => {
                // A "real" node should probably "lock" the UTXOs spent in funding transactions until
                // the funding transaction either confirms, or this event is generated.
            }
        }
    }
}
