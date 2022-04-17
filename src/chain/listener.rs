use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::node::{ChainMonitor, ChannelManager};
use bitcoin::{Block, BlockHeader};
use lightning::chain::Listen;

use super::listener_database::ListenerDatabase;

type Listener = (Arc<ChainMonitor>, Arc<ChannelManager>, ListenerDatabase);

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
    fn block_connected(&self, block: &Block, height: u32) {
        let listeners = self.listeners.lock().unwrap();
        for (chain_monitor, channel_manager, listener_database) in listeners.values() {
            channel_manager.block_connected(block, height);
            chain_monitor.block_connected(block, height);
            listener_database.block_connected(block, height);
        }
    }

    fn block_disconnected(&self, header: &BlockHeader, height: u32) {
        let listeners = self.listeners.lock().unwrap();
        for (chain_monitor, channel_manager, listener_database) in listeners.values() {
            channel_manager.block_disconnected(header, height);
            chain_monitor.block_disconnected(header, height);
            listener_database.block_disconnected(header, height);
        }
    }
}
