use bitcoin::{Script, Txid, secp256k1::PublicKey};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum SenseiEvent {
    TransactionBroadcast {
        node_id: String,
        txid: Txid,
    },
    FundingGenerationReady {
        node_id: String,
        temporary_channel_id: [u8; 32],
        channel_value_satoshis: u64,
        output_script: Script,
        user_channel_id: u64,
        counterparty_node_id: PublicKey
    },
}
