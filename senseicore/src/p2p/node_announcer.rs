use lightning::ln::msgs::NetAddress;

use crate::node::PeerManager;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::Duration,
};

pub type AnnouncementNodeDetails = (Arc<PeerManager>, Vec<NetAddress>, [u8; 32]);

pub enum NodeAnnouncerRequest {
    RegisterNode(String, Arc<PeerManager>, Vec<NetAddress>, [u8; 32]),
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
        peer_manager: Arc<PeerManager>,
        listen_addresses: Vec<NetAddress>,
        alias: [u8; 32],
    ) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(NodeAnnouncerRequest::RegisterNode(
            id,
            peer_manager,
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
            for (_node_id, (peer_manager, listen_addresses, alias)) in nodes.iter() {
                if !listen_addresses.is_empty() {
                    peer_manager.broadcast_node_announcement(
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
                        peer_manager,
                        listen_addresses,
                        alias,
                    ) => {
                        nodes.insert(id, (peer_manager, listen_addresses, alias));
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
