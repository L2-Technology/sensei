use anyhow::{anyhow, Result};
use bitcoin::{Script, Transaction, TxIn, TxOut, Witness};
use lightning::chain::keysinterface::{
    DelayedPaymentOutputDescriptor, SpendableOutputDescriptor, StaticPaymentOutputDescriptor,
};
use lightning_signer::bitcoin;
use lightning_signer::util::transaction_utils::{self, MAX_VALUE_MSAT};
use std::collections::HashSet;
use triggered::{Listener, Trigger};

pub fn create_spending_transaction(
    descriptors: &[&SpendableOutputDescriptor],
    outputs: Vec<TxOut>,
    change_destination_script: Script,
    feerate_sat_per_1000_weight: u32,
) -> Result<Transaction> {
    let mut input = Vec::new();
    let mut input_value = 0;
    let mut witness_weight = 0;
    let mut output_set = HashSet::with_capacity(descriptors.len());
    for outp in descriptors {
        match outp {
            SpendableOutputDescriptor::StaticPaymentOutput(descriptor) => {
                input.push(TxIn {
                    previous_output: descriptor.outpoint.into_bitcoin_outpoint(),
                    script_sig: Script::new(),
                    sequence: 0,
                    witness: Witness::default(),
                });
                witness_weight += StaticPaymentOutputDescriptor::MAX_WITNESS_LENGTH;
                input_value += descriptor.output.value;
                if !output_set.insert(descriptor.outpoint) {
                    return Err(anyhow!("duplicate"));
                }
            }
            SpendableOutputDescriptor::DelayedPaymentOutput(descriptor) => {
                input.push(TxIn {
                    previous_output: descriptor.outpoint.into_bitcoin_outpoint(),
                    script_sig: Script::new(),
                    sequence: descriptor.to_self_delay as u32,
                    witness: Witness::default(),
                });
                witness_weight += DelayedPaymentOutputDescriptor::MAX_WITNESS_LENGTH;
                input_value += descriptor.output.value;
                if !output_set.insert(descriptor.outpoint) {
                    return Err(anyhow!("duplicate"));
                }
            }
            SpendableOutputDescriptor::StaticOutput {
                ref outpoint,
                ref output,
            } => {
                input.push(TxIn {
                    previous_output: outpoint.into_bitcoin_outpoint(),
                    script_sig: Script::new(),
                    sequence: 0,
                    witness: Witness::default(),
                });
                witness_weight += 1 + 73 + 34;
                input_value += output.value;
                if !output_set.insert(*outpoint) {
                    return Err(anyhow!("duplicate"));
                }
            }
        }
        if input_value > MAX_VALUE_MSAT / 1000 {
            return Err(anyhow!("overflow"));
        }
    }
    let mut spend_tx = Transaction {
        version: 2,
        lock_time: 0,
        input,
        output: outputs,
    };
    transaction_utils::maybe_add_change_output(
        &mut spend_tx,
        input_value,
        witness_weight,
        feerate_sat_per_1000_weight,
        change_destination_script,
    )
    .map_err(|()| anyhow!("could not add change"))?;
    Ok(spend_tx)
}

#[derive(Clone)]
pub struct Shutter {
    pub trigger: Trigger,
    pub signal: Listener,
}

impl Shutter {
    /// There should only be one of these per process
    pub fn new() -> Self {
        let (trigger, signal) = triggered::trigger();
        let ctrlc_trigger = trigger.clone();
        // ctrlc::set_handler(move || {
        // 	println!("got termination signal");
        // 	ctrlc_trigger.trigger();
        // })
        // .expect("Error setting Ctrl-C handler - do you have more than one?");

        Self { trigger, signal }
    }
}
