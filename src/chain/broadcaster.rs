use std::sync::Arc;

use bitcoin::Transaction;
use lightning::chain::chaininterface::BroadcasterInterface;

use super::{bitcoind_client::BitcoindClient, listener_database::ListenerDatabase};

pub struct SenseiBroadcaster {
    pub bitcoind_client: Arc<BitcoindClient>,
    pub listener_database: ListenerDatabase,
}

impl BroadcasterInterface for SenseiBroadcaster {
    fn broadcast_transaction(&self, tx: &Transaction) {
        self.bitcoind_client.broadcast_transaction(tx);

        // TODO: there's a bug here if the broadcast fails
        //       best solution is to probably setup a zmq listener
        self.listener_database.process_mempool_tx(tx);
    }
}
