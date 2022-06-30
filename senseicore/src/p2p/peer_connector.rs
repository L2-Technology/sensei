use bitcoin::secp256k1::PublicKey;
use lightning::{ln::msgs::NetAddress, routing::gossip::NodeId};

use crate::node::{ChannelManager, NetworkGraph, PeerManager, RoutingPeerManager};
use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};

pub enum PeerConnectorRequest {
    RegisterNode(String, Arc<PeerManager>, Arc<ChannelManager>),
    UnregisterNode(String),
}

pub struct PeerConnector {
    pub routing_peer_manager: Arc<RoutingPeerManager>,
    pub requests: Arc<Mutex<VecDeque<PeerConnectorRequest>>>,
    pub network_graph: Arc<NetworkGraph>,
    pub interval: Duration,
    pub target_routing_peers: u16,
    pub pubkey_addr_map: Arc<Mutex<HashMap<PublicKey, SocketAddr>>>,
}

impl PeerConnector {
    pub fn new(
        network_graph: Arc<NetworkGraph>,
        routing_peer_manager: Arc<RoutingPeerManager>,
    ) -> Self {
        Self {
            routing_peer_manager,
            requests: Arc::new(Mutex::new(VecDeque::new())),
            network_graph,
            interval: Duration::from_secs(5),
            target_routing_peers: 10,
            pubkey_addr_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect_routing_peer(
        &self,
        pubkey: PublicKey,
        peer_addr: SocketAddr,
    ) -> Result<(), ()> {
        match lightning_net_tokio::connect_outbound(
            Arc::clone(&self.routing_peer_manager),
            pubkey,
            peer_addr,
        )
        .await
        {
            Some(connection_closed_future) => {
                let mut connection_closed_future = Box::pin(connection_closed_future);
                loop {
                    match futures::poll!(&mut connection_closed_future) {
                        std::task::Poll::Ready(_) => {
                            println!("ERROR: Peer disconnected before we finished the handshake");
                            return Err(());
                        }
                        std::task::Poll::Pending => {}
                    }
                    // Avoid blocking the tokio context by sleeping a bit
                    match self
                        .routing_peer_manager
                        .get_peer_node_ids()
                        .iter()
                        .find(|id| **id == pubkey)
                    {
                        Some(_) => break,
                        None => tokio::time::sleep(Duration::from_millis(10)).await,
                    }
                }
            }
            None => {
                //println!("ERROR: failed to connect to peer");
                return Err(());
            }
        }
        Ok(())
    }

    pub async fn connect_peer(
        &self,
        pubkey: PublicKey,
        peer_addr: SocketAddr,
        peer_manager: Arc<PeerManager>,
    ) -> Result<(), ()> {
        match lightning_net_tokio::connect_outbound(Arc::clone(&peer_manager), pubkey, peer_addr)
            .await
        {
            Some(connection_closed_future) => {
                let mut connection_closed_future = Box::pin(connection_closed_future);
                loop {
                    match futures::poll!(&mut connection_closed_future) {
                        std::task::Poll::Ready(_) => {
                            println!("ERROR: Peer disconnected before we finished the handshake");
                            return Err(());
                        }
                        std::task::Poll::Pending => {}
                    }
                    // Avoid blocking the tokio context by sleeping a bit
                    match peer_manager
                        .get_peer_node_ids()
                        .iter()
                        .find(|id| **id == pubkey)
                    {
                        Some(_) => break,
                        None => tokio::time::sleep(Duration::from_millis(10)).await,
                    }
                }
                {
                    let mut pubkey_addr_map = self.pubkey_addr_map.lock().unwrap();
                    pubkey_addr_map.insert(pubkey, peer_addr);
                }
                if self.router_should_connect_to_peer(&pubkey) {
                    let _res = self.connect_routing_peer(pubkey, peer_addr).await;
                }
            }
            None => {
                //println!("ERROR: failed to connect to peer");
                return Err(());
            }
        }
        Ok(())
    }

    pub fn router_needs_more_peers(&self) -> bool {
        self.routing_peer_manager.get_peer_node_ids().len() < self.target_routing_peers.into()
    }

    pub fn router_should_connect_to_peer(&self, pubkey: &PublicKey) -> bool {
        let peers = self.routing_peer_manager.get_peer_node_ids();
        peers.len() < self.target_routing_peers.into() && !peers.contains(pubkey)
    }

    pub fn router_connected_to_peer(&self, pubkey: PublicKey) -> bool {
        self.routing_peer_manager
            .get_peer_node_ids()
            .contains(&pubkey)
    }

    pub async fn connect_peer_if_necessary(
        &self,
        pubkey: PublicKey,
        peer_addr: SocketAddr,
        peer_manager: Arc<PeerManager>,
    ) -> Result<(), ()> {
        if !peer_manager.get_peer_node_ids().contains(&pubkey) {
            self.connect_peer(pubkey, peer_addr, peer_manager).await?
        }
        Ok(())
    }

    pub fn register_node(
        &self,
        id: String,
        peer_manager: Arc<PeerManager>,
        channel_manager: Arc<ChannelManager>,
    ) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(PeerConnectorRequest::RegisterNode(
            id,
            peer_manager,
            channel_manager,
        ));
    }

    pub fn unregister_node(&self, id: String) {
        let mut requests = self.requests.lock().unwrap();
        requests.push_back(PeerConnectorRequest::UnregisterNode(id));
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

            // if routing peers disconnected then try to add some more
            let mut routing_peer_node_ids = self.routing_peer_manager.get_peer_node_ids();
            if routing_peer_node_ids.len() < self.target_routing_peers.into() {
                for (peer_manager, _channel_manager) in nodes.values() {
                    let peer_node_ids = peer_manager.get_peer_node_ids();
                    for peer_node_id in peer_node_ids.into_iter() {
                        if routing_peer_node_ids.len() < self.target_routing_peers.into()
                            && !routing_peer_node_ids.contains(&peer_node_id)
                        {
                            let peer_addr_opt = {
                                let peer_addr_map = self.pubkey_addr_map.lock().unwrap();
                                peer_addr_map.get(&peer_node_id).copied()
                            };

                            if let Some(peer_addr) = peer_addr_opt {
                                if let Ok(()) =
                                    self.connect_routing_peer(peer_node_id, peer_addr).await
                                {
                                    routing_peer_node_ids.push(peer_node_id);
                                }
                            }
                        }
                    }

                    if routing_peer_node_ids.len() >= self.target_routing_peers.into() {
                        break;
                    }
                }
            }

            let mut requests = self.requests.lock().unwrap();
            while let Some(request) = requests.pop_front() {
                match request {
                    PeerConnectorRequest::RegisterNode(id, peer_manager, channel_manager) => {
                        nodes.insert(id, (peer_manager, channel_manager));
                    }
                    PeerConnectorRequest::UnregisterNode(id) => {
                        nodes.remove(&id);
                    }
                }
            }
        }
    }
}
