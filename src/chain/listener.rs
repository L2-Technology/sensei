use crate::node::{ChainMonitor, ChannelManager};
use bitcoin::{Block, BlockHeader};
use lightning::chain::Listen;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    runtime::Handle,
    sync::{Mutex, MutexGuard},
    task,
};

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

    pub fn get_listeners(&self) -> MutexGuard<HashMap<String, Listener>> {
        task::block_in_place(move || {
            Handle::current().block_on(async move { self.listeners.lock().await })
        })
    }

    fn get_key(&self, listener: &Listener) -> String {
        listener.1.get_our_node_id().to_string()
    }

    pub async fn add_listener(&self, listener: Listener) {
        let mut listeners = self.listeners.lock().await;
        listeners.insert(self.get_key(&listener), listener);
    }

    pub async fn remove_listener(&self, listener: Listener) {
        let mut listeners = self.listeners.lock().await;
        listeners.remove(&self.get_key(&listener));
    }
}

impl Listen for SenseiChainListener {
    fn block_connected(&self, block: &Block, height: u32) {
        let listeners = self.get_listeners();
        for (chain_monitor, channel_manager, listener_database) in listeners.values() {
            channel_manager.block_connected(block, height);
            chain_monitor.block_connected(block, height);
            listener_database.block_connected(block, height);
        }
    }

    fn block_disconnected(&self, header: &BlockHeader, height: u32) {
        let listeners = self.get_listeners();
        for (chain_monitor, channel_manager, listener_database) in listeners.values() {
            channel_manager.block_disconnected(header, height);
            chain_monitor.block_disconnected(header, height);
            listener_database.block_disconnected(header, height);
        }
    }
}
