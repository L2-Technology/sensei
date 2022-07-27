use lightning::ln::msgs::NetAddress;

use crate::node::ChannelManager;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Duration,
};

pub type AnnouncementNodeDetails = (Arc<ChannelManager>, Vec<NetAddress>, [u8; 32]);

pub enum NodeAnnouncerRequest {
    RegisterNode(String, Arc<ChannelManager>, Vec<NetAddress>, [u8; 32]),
    UnregisterNode(String),
}

pub struct NodeAnnouncer {
    pub requests: Arc<Mutex<VecDeque<NodeAnnouncerRequest>>>,
    pub interval: Duration,
}

impl NodeAnnouncer {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(VecDeque::new())),
            interval: Duration::from_secs(60),
        }
    }

    pub fn register_node(
        &self,
        id: String,
        channel_manager: Arc<ChannelManager>,
        listen_addresses: Vec<NetAddress>,
        alias: [u8; 32],
    ) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(NodeAnnouncerRequest::RegisterNode(
            id,
            channel_manager,
            listen_addresses,
            alias,
        ));
    }

    pub fn unregister_node(&self, id: String) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(NodeAnnouncerRequest::UnregisterNode(id));
    }

    pub async fn run(&self) {
        let mut nodes: HashMap<String, AnnouncementNodeDetails> = HashMap::new();
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            for (_node_id, (channel_manager, listen_addresses, alias)) in nodes.iter() {
                if !listen_addresses.is_empty() {
                    channel_manager.broadcast_node_announcement(
                        [0; 3],
                        *alias,
                        listen_addresses.clone(),
                    );
                }
            }

            let mut requests = self.requests.lock().unwrap();
            while let Some(request) = requests.pop_front() {
                match request {
                    NodeAnnouncerRequest::RegisterNode(
                        id,
                        channel_manager,
                        listen_addresses,
                        alias,
                    ) => {
                        nodes.insert(id, (channel_manager, listen_addresses, alias));
                    }
                    NodeAnnouncerRequest::UnregisterNode(id) => {
                        nodes.remove(&id);
                    }
                }
            }
        }
    }
}

impl Default for NodeAnnouncer {
    fn default() -> Self {
        Self::new()
    }
}
