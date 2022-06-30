use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};

use lightning::{ln::msgs::NetAddress, routing::gossip::NodeId};

use crate::node::{ChannelManager, NetworkGraph, PeerManager};

use super::peer_connector::PeerConnector;

pub enum ChannelPeerReconnectorRequest {
    RegisterNode(String, Arc<PeerManager>, Arc<ChannelManager>),
    UnregisterNode(String),
}

pub struct ChannelPeerReconnector {
    pub requests: Mutex<VecDeque<ChannelPeerReconnectorRequest>>,
    pub network_graph: Arc<NetworkGraph>,
    pub interval: Duration,
    pub peer_connector: Arc<PeerConnector>,
}

impl ChannelPeerReconnector {
    pub fn new(peer_connector: Arc<PeerConnector>, network_graph: Arc<NetworkGraph>) -> Self {
        Self {
            network_graph,
            interval: Duration::from_secs(5),
            requests: Mutex::new(VecDeque::new()),
            peer_connector,
        }
    }

    pub fn register_node(
        &self,
        id: String,
        peer_manager: Arc<PeerManager>,
        channel_manager: Arc<ChannelManager>,
    ) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(ChannelPeerReconnectorRequest::RegisterNode(
            id,
            peer_manager,
            channel_manager,
        ));
    }

    pub fn unregister_node(&self, id: String) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(ChannelPeerReconnectorRequest::UnregisterNode(id));
    }

    pub async fn run(&self) {
        let mut nodes: HashMap<String, (Arc<PeerManager>, Arc<ChannelManager>)> = HashMap::new();
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;

            for (peer_manager, channel_manager) in nodes.values() {
                for chan_info in channel_manager.list_channels() {
                    let pubkey = chan_info.counterparty.node_id;
                    if !chan_info.is_usable && !peer_manager.get_peer_node_ids().contains(&pubkey) {
                        let node_id = NodeId::from_pubkey(&pubkey);
                        let addresses = {
                            let network_graph = self.network_graph.read_only();
                            network_graph.nodes().get(&node_id).and_then(|info| {
                                info.announcement_info
                                    .as_ref()
                                    .map(|info| info.addresses.clone())
                            })
                        };

                        if let Some(addresses) = addresses {
                            for address in addresses {
                                let addr = match address {
                                    NetAddress::IPv4 { addr, port } => {
                                        Some(SocketAddr::new(IpAddr::from(addr), port))
                                    }
                                    NetAddress::IPv6 { addr, port } => {
                                        Some(SocketAddr::new(IpAddr::from(addr), port))
                                    }
                                    NetAddress::OnionV2(_) => None,
                                    NetAddress::OnionV3 { .. } => None,
                                };

                                if let Some(addr) = addr {
                                    if let Ok(()) = self
                                        .peer_connector
                                        .connect_peer_if_necessary(
                                            pubkey,
                                            addr,
                                            peer_manager.clone(),
                                        )
                                        .await
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut requests = self.requests.lock().unwrap();
            while let Some(request) = requests.pop_front() {
                match request {
                    ChannelPeerReconnectorRequest::RegisterNode(
                        id,
                        peer_manager,
                        channel_manager,
                    ) => {
                        nodes.insert(id, (peer_manager, channel_manager));
                    }
                    ChannelPeerReconnectorRequest::UnregisterNode(id) => {
                        nodes.remove(&id);
                    }
                }
            }
        }
    }
}
