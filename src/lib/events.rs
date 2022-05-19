use bitcoin::Txid;

#[derive(Clone, Debug)]
pub enum SenseiEvent {
    TransactionBroadcast { node_id: String, txid: Txid },
    Ldk(Box<lightning::util::events::Event>),
}
