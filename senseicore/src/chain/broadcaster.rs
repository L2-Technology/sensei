use std::sync::{Arc, Mutex};

use crate::events::SenseiEvent;

use super::database::WalletDatabase;
use bitcoin::Transaction;
use lightning::chain::chaininterface::BroadcasterInterface;
use tokio::sync::broadcast;

pub struct SenseiBroadcaster {
    pub node_id: String,
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    pub wallet_database: Arc<Mutex<WalletDatabase>>,
    pub event_sender: broadcast::Sender<SenseiEvent>,
}

impl BroadcasterInterface for SenseiBroadcaster {
    fn broadcast_transaction(&self, tx: &Transaction) {
        self.broadcaster.broadcast_transaction(tx);

        // TODO: there's a bug here if the broadcast fails
        //       best solution is to probably setup a zmq listener
        let mut database = self.wallet_database.lock().unwrap();
        database.process_mempool_tx(tx);

        self.event_sender
            .send(SenseiEvent::TransactionBroadcast {
                node_id: self.node_id.clone(),
                txid: tx.txid(),
            })
            .unwrap_or_default();
    }
}
