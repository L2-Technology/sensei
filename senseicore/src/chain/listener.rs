use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::node::{ChainMonitor, ChannelManager};
use bitcoin::BlockHeader;
use lightning::chain::transaction::TransactionData;
use lightning::chain::Listen;

use super::database::WalletDatabase;

type Listener = (Arc<ChainMonitor>, Arc<ChannelManager>, WalletDatabase);

pub struct SenseiChainListener {
    listeners: Mutex<HashMap<String, Listener>>,
}

impl Default for SenseiChainListener {
    fn default() -> Self {
        Self::new()
    }
}

impl SenseiChainListener {
    pub fn new() -> Self {
        Self {
            listeners: Mutex::new(HashMap::new()),
        }
    }

    fn get_key(&self, listener: &Listener) -> String {
        listener.1.get_our_node_id().to_string()
    }

    pub fn add_listener(&self, listener: Listener) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.insert(self.get_key(&listener), listener);
    }

    pub fn remove_listener(&self, listener: Listener) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.remove(&self.get_key(&listener));
    }
}

impl Listen for SenseiChainListener {
    fn filtered_block_connected(
        &self,
        header: &BlockHeader,
        txdata: &TransactionData,
        height: u32,
    ) {
        let listeners = self.listeners.lock().unwrap();
        for (chain_monitor, channel_manager, wallet_database) in listeners.values() {
            channel_manager.filtered_block_connected(header, txdata, height);
            chain_monitor.filtered_block_connected(header, txdata, height);
            wallet_database.filtered_block_connected(header, txdata, height);
        }
    }

    fn block_disconnected(&self, header: &BlockHeader, height: u32) {
        let listeners = self.listeners.lock().unwrap();
        for (chain_monitor, channel_manager, wallet_database) in listeners.values() {
            channel_manager.block_disconnected(header, height);
            chain_monitor.block_disconnected(header, height);
            wallet_database.block_disconnected(header, height);
        }
    }
}
