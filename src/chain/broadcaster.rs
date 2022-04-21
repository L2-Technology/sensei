use std::sync::{Arc, Mutex};

use super::database::WalletDatabase;
use bitcoin::Transaction;
use lightning::chain::chaininterface::BroadcasterInterface;

pub struct SenseiBroadcaster {
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    pub wallet_database: Arc<Mutex<WalletDatabase>>,
}

impl BroadcasterInterface for SenseiBroadcaster {
    fn broadcast_transaction(&self, tx: &Transaction) {
        self.broadcaster.broadcast_transaction(tx);

        // TODO: there's a bug here if the broadcast fails
        //       best solution is to probably setup a zmq listener
        let mut database = self.wallet_database.lock().unwrap();
        database.process_mempool_tx(tx);
    }
}
