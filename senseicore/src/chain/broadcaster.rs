use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::events::SenseiEvent;

use super::database::WalletDatabase;
use bitcoin::{Transaction, Txid};
use lightning::chain::chaininterface::BroadcasterInterface;
use tokio::sync::broadcast;

pub struct SenseiBroadcaster {
    pub debounce: Mutex<HashMap<Txid, usize>>,
    pub node_id: String,
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    pub wallet_database: Arc<Mutex<WalletDatabase>>,
    pub event_sender: broadcast::Sender<SenseiEvent>,
}

impl SenseiBroadcaster {
    pub fn new(
        node_id: String,
        broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
        wallet_database: Arc<Mutex<WalletDatabase>>,
        event_sender: broadcast::Sender<SenseiEvent>,
    ) -> Self {
        Self {
            node_id,
            broadcaster,
            wallet_database,
            event_sender,
            debounce: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_debounce(&self, txid: Txid, count: usize) {
        let mut debounce = self.debounce.lock().unwrap();
        debounce.insert(txid, count);
    }

    pub fn broadcast(&self, tx: &Transaction) {
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

impl BroadcasterInterface for SenseiBroadcaster {
    fn broadcast_transaction(&self, tx: &Transaction) {
        let txid = tx.txid();

        let mut debounce = self.debounce.lock().unwrap();

        let can_broadcast = match debounce.get_mut(&txid) {
            Some(count) => {
                *count -= 1;
                *count == 0
            }
            None => true,
        };

        if can_broadcast {
            self.broadcast(tx);
        }
    }
}
