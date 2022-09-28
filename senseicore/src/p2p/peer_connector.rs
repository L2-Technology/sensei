use bitcoin::secp256k1::PublicKey;
use entity::{
    peer_address::PeerAddressSource,
    sea_orm::{ActiveModelTrait, ActiveValue},
    seconds_since_epoch,
};
use lightning::{
    ln::msgs::NetAddress,
    routing::gossip::NodeId,
    util::ser::{Readable, Writeable},
};

use crate::{
    database::SenseiDatabase,
    error::Error,
    node::{ChannelManager, PeerManager, RoutingPeerManager},
};
use std::{
    collections::{HashMap, VecDeque},
    io::Cursor,
    sync::{Arc, Mutex},
    time::Duration,
};

use super::{node_info::NodeInfoLookup, utils::net_address_to_socket_addr};

pub enum PeerConnectorRequest {
    RegisterNode(String, Arc<PeerManager>, Arc<ChannelManager>),
    UnregisterNode(String),
}

pub struct PeerConnector {
    pub database: Arc<SenseiDatabase>,
    pub routing_peer_manager: Option<Arc<RoutingPeerManager>>,
    pub requests: Arc<Mutex<VecDeque<PeerConnectorRequest>>>,
    pub node_info_lookup: Arc<NodeInfoLookup>,
    pub interval: Duration,
    pub target_routing_peers: u16,
}

impl PeerConnector {
    pub fn new(
        database: Arc<SenseiDatabase>,
        node_info_lookup: Arc<NodeInfoLookup>,
        routing_peer_manager: Option<Arc<RoutingPeerManager>>,
    ) -> Self {
        Self {
            database,
            routing_peer_manager,
            requests: Arc::new(Mutex::new(VecDeque::new())),
            node_info_lookup,
            interval: Duration::from_secs(5),
            target_routing_peers: 10,
        }
    }

    pub async fn get_addresses_for_pubkey(
        &self,
        node_id: &str,
        pubkey: &PublicKey,
    ) -> Result<Vec<NetAddress>, Error> {
        let target_node_id = NodeId::from_pubkey(pubkey);

        let mut network_graph_addresses = self.node_info_lookup.get_addresses(target_node_id)?;

        let mut database_addresses = self
            .database
            .list_peer_addresses(node_id, &pubkey.to_string()[..])
            .await?
            .into_iter()
            .map(|peer_address| {
                let mut readable_peer_address = Cursor::new(peer_address.address);
                NetAddress::read(&mut readable_peer_address).unwrap()
            })
            .collect::<Vec<_>>();

        database_addresses.append(&mut network_graph_addresses);

        // TODO: filter duplicates?

        Ok(database_addresses)
    }

    pub async fn connect_routing_peer(
        &self,
        pubkey: PublicKey,
        peer_addr: NetAddress,
    ) -> Result<(), ()> {
        if let Some(routing_peer_manager) = &self.routing_peer_manager {
            let socket_addr = net_address_to_socket_addr(peer_addr.clone());
            if socket_addr.is_none() {
                return Err(());
            }
            let socket_addr = socket_addr.unwrap();

            match lightning_net_tokio::connect_outbound(
                Arc::clone(routing_peer_manager),
                pubkey,
                socket_addr,
            )
            .await
            {
                Some(connection_closed_future) => {
                    let mut connection_closed_future = Box::pin(connection_closed_future);
                    loop {
                        match futures::poll!(&mut connection_closed_future) {
                            std::task::Poll::Ready(_) => {
                                println!(
                                    "ERROR: Peer disconnected before we finished the handshake"
                                );
                                return Err(());
                            }
                            std::task::Poll::Pending => {}
                        }
                        // Avoid blocking the tokio context by sleeping a bit
                        match routing_peer_manager
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
                    return Err(());
                }
            }
        }

        Ok(())
    }

    pub async fn connect_peer(
        &self,
        local_node_id: &str,
        pubkey: PublicKey,
        peer_addr: NetAddress,
        peer_manager: Arc<PeerManager>,
    ) -> Result<(), ()> {
        let socket_addr = net_address_to_socket_addr(peer_addr.clone());
        if socket_addr.is_none() {
            return Err(());
        }
        let socket_addr = socket_addr.unwrap();
        match lightning_net_tokio::connect_outbound(Arc::clone(&peer_manager), pubkey, socket_addr)
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

                match self
                    .database
                    .list_peer_addresses(local_node_id, &pubkey.to_string())
                    .await
                {
                    Ok(known_addresses) => {
                        let known_address = known_addresses.iter().find(|address| {
                            let mut existing_address_readable =
                                Cursor::new(address.address.clone());
                            let address = NetAddress::read(&mut existing_address_readable).unwrap();
                            address == peer_addr
                        });

                        let now: i64 = seconds_since_epoch();

                        let result = match known_address {
                            Some(address) => {
                                let mut peer_address: entity::peer_address::ActiveModel =
                                    address.clone().into();
                                peer_address.last_connected_at = ActiveValue::Set(now);
                                peer_address.update(self.database.get_connection()).await
                            }
                            None => {
                                let peer_address = entity::peer_address::ActiveModel {
                                    node_id: ActiveValue::Set(local_node_id.to_string()),
                                    pubkey: ActiveValue::Set(pubkey.to_string()),
                                    last_connected_at: ActiveValue::Set(now),
                                    address: ActiveValue::Set(peer_addr.encode()),
                                    source: ActiveValue::Set(
                                        PeerAddressSource::OutboundConnect.into(),
                                    ),
                                    ..Default::default()
                                };
                                peer_address.insert(self.database.get_connection()).await
                            }
                        };

                        if let Err(e) = result {
                            println!("failed to update list peers database: {:?}", e);
                        }
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
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
        match &self.routing_peer_manager {
            Some(routing_peer_manager) => {
                routing_peer_manager.get_peer_node_ids().len() < self.target_routing_peers.into()
            }
            None => false,
        }
    }

    pub fn router_should_connect_to_peer(&self, pubkey: &PublicKey) -> bool {
        match &self.routing_peer_manager {
            Some(routing_peer_manager) => {
                let peers = routing_peer_manager.get_peer_node_ids();
                peers.len() < self.target_routing_peers.into() && !peers.contains(pubkey)
            }
            None => false,
        }
    }

    pub fn router_connected_to_peer(&self, pubkey: PublicKey) -> bool {
        match &self.routing_peer_manager {
            Some(routing_peer_manager) => {
                routing_peer_manager.get_peer_node_ids().contains(&pubkey)
            }
            None => false,
        }
    }

    pub async fn connect_peer_if_necessary(
        &self,
        local_node_id: &str,
        pubkey: PublicKey,
        peer_addr: NetAddress,
        peer_manager: Arc<PeerManager>,
    ) -> Result<(), ()> {
        if !peer_manager.get_peer_node_ids().contains(&pubkey) {
            self.connect_peer(local_node_id, pubkey, peer_addr, peer_manager)
                .await?
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
            for (node_id, (peer_manager, channel_manager)) in nodes.iter() {
                for chan_info in channel_manager.list_channels() {
                    let pubkey = chan_info.counterparty.node_id;
                    if !chan_info.is_usable && !peer_manager.get_peer_node_ids().contains(&pubkey) {
                        if let Ok(addresses) =
                            self.get_addresses_for_pubkey(&node_id[..], &pubkey).await
                        {
                            for address in addresses {
                                if let Ok(()) = self
                                    .connect_peer_if_necessary(
                                        node_id,
                                        pubkey,
                                        address,
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

            // if routing peers disconnected then try to add some more
            if let Some(routing_peer_manager) = &self.routing_peer_manager {
                let mut routing_peer_node_ids = routing_peer_manager.get_peer_node_ids();
                if routing_peer_node_ids.len() < self.target_routing_peers.into() {
                    for (node_id, (peer_manager, _channel_manager)) in nodes.iter() {
                        let peer_node_ids = peer_manager.get_peer_node_ids();
                        for peer_node_id in peer_node_ids.into_iter() {
                            if routing_peer_node_ids.len() < self.target_routing_peers.into()
                                && !routing_peer_node_ids.contains(&peer_node_id)
                            {
                                if let Ok(peer_addresses) =
                                    self.get_addresses_for_pubkey(node_id, &peer_node_id).await
                                {
                                    for peer_addr in peer_addresses {
                                        if let Ok(()) =
                                            self.connect_routing_peer(peer_node_id, peer_addr).await
                                        {
                                            routing_peer_node_ids.push(peer_node_id);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if routing_peer_node_ids.len() >= self.target_routing_peers.into() {
                            break;
                        }
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
