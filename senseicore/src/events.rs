use bitcoin::Txid;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum SenseiEvent {
    TransactionBroadcast { node_id: String, txid: Txid },
}
