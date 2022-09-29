// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.
pub mod background_processor;
pub mod bubble_gossip_route_handler;
pub mod node_announcer;
pub mod node_info;
pub mod peer_connector;
pub mod router;
pub mod utils;

use lightning::{
    chain::{
        self,
        keysinterface::{KeysInterface, KeysManager, Recipient},
    },
    ln::peer_handler::{ErroringMessageHandler, IgnoringMessageHandler, MessageHandler},
};

use lightning_invoice::utils::DefaultRouter;
use rand::RngCore;
use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::SystemTime,
};
use tokio::task::JoinHandle;

use crate::{
    config::{P2PConfig, SenseiConfig},
    database::SenseiDatabase,
    disk::FilesystemLogger,
    node::{LightningNode, NetworkGraph, NetworkGraphMessageHandler, RoutingPeerManager},
    persist::{AnyKVStore, DatabaseStore, SenseiPersister},
};

use self::{
    background_processor::BackgroundProcessor,
    bubble_gossip_route_handler::AnyP2PGossipHandler,
    node_announcer::NodeAnnouncer,
    node_info::NodeInfoLookup,
    peer_connector::PeerConnector,
    router::{AnyRouter, AnyScorer},
    utils::parse_peer_info,
};

pub struct SenseiP2P {
    pub config: Arc<SenseiConfig>,
    pub persister: Arc<SenseiPersister>,
    pub network_graph: Arc<NetworkGraph>,
    pub p2p_gossip: Arc<AnyP2PGossipHandler>,
    pub scorer: Arc<Mutex<AnyScorer>>,
    pub logger: Arc<FilesystemLogger>,
    pub peer_manager: Option<Arc<RoutingPeerManager>>,
    pub peer_connector: Arc<PeerConnector>,
    pub node_announcer: Arc<NodeAnnouncer>,
    pub runtime_handle: tokio::runtime::Handle,
    pub stop_signal: Arc<AtomicBool>,
    pub join_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl SenseiP2P {
    pub async fn new(
        config: Arc<SenseiConfig>,
        database: Arc<SenseiDatabase>,
        logger: Arc<FilesystemLogger>,
        runtime_handle: tokio::runtime::Handle,
        stop_signal: Arc<AtomicBool>,
    ) -> Self {
        let p2p_node_id = config.instance_name.clone();

        let persistence_store =
            AnyKVStore::Database(DatabaseStore::new(database.clone(), p2p_node_id.clone()));

        let persister = Arc::new(SenseiPersister::new(
            persistence_store,
            config.network,
            logger.clone(),
        ));

        let network_graph = Arc::new(persister.read_network_graph());

        let scorer = match config.get_p2p_config() {
            P2PConfig::Remote(host, token) => Arc::new(Mutex::new(AnyScorer::new_remote(
                host,
                token,
                runtime_handle.clone(),
            ))),
            _ => Arc::new(Mutex::new(AnyScorer::Local(
                persister.read_scorer(Arc::clone(&network_graph)),
            ))),
        };

        let node_info_lookup = match config.get_p2p_config() {
            P2PConfig::Remote(host, token) => Arc::new(NodeInfoLookup::new_remote(
                host,
                token,
                runtime_handle.clone(),
            )),
            _ => Arc::new(NodeInfoLookup::Local(network_graph.clone())),
        };

        let p2p_gossip = match config.get_p2p_config() {
            P2PConfig::Remote(host, token) => Arc::new(AnyP2PGossipHandler::new_remote(
                host,
                token,
                runtime_handle.clone(),
            )),
            P2PConfig::RapidGossipSync(_) => Arc::new(AnyP2PGossipHandler::None),
            P2PConfig::Local => {
                Arc::new(AnyP2PGossipHandler::Local(NetworkGraphMessageHandler::new(
                    Arc::clone(&network_graph),
                    None::<Arc<dyn chain::Access + Send + Sync>>,
                    logger.clone(),
                )))
            }
        };

        let lightning_msg_handler = MessageHandler {
            chan_handler: Arc::new(ErroringMessageHandler::new()),
            route_handler: p2p_gossip.clone(),
        };

        let mut entropy: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut entropy);

        match database.get_entropy_sync(p2p_node_id.clone()).unwrap() {
            Some(entropy_vec) => {
                entropy.copy_from_slice(entropy_vec.as_slice());
            }
            None => {
                let _res = database.set_entropy_sync(p2p_node_id, entropy.to_vec());
            }
        }

        let seed = LightningNode::get_seed_from_entropy(config.network, &entropy);

        let cur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let keys_manager = Arc::new(KeysManager::new(&seed, cur.as_secs(), cur.subsec_nanos()));

        let mut ephemeral_bytes = [0; 32];
        rand::thread_rng().fill_bytes(&mut ephemeral_bytes);

        let peer_manager = match config.get_p2p_config() {
            P2PConfig::RapidGossipSync(_) => None,
            _ => Some(Arc::new(RoutingPeerManager::new(
                lightning_msg_handler,
                keys_manager.get_node_secret(Recipient::Node).unwrap(),
                &ephemeral_bytes,
                logger.clone(),
                Arc::new(IgnoringMessageHandler {}),
            ))),
        };

        let peer_connector = Arc::new(PeerConnector::new(
            database,
            node_info_lookup,
            peer_manager.clone(),
        ));

        for gossip_peer in config.gossip_peers.split(',') {
            if let Ok((pubkey, peer_addr)) = parse_peer_info(gossip_peer.to_string()).await {
                let _res = peer_connector.connect_routing_peer(pubkey, peer_addr).await;
            }
        }

        let node_announcer = Arc::new(NodeAnnouncer::new());

        let p2p_background_processor = BackgroundProcessor::new(
            peer_manager.clone(),
            scorer.clone(),
            network_graph.clone(),
            persister.clone(),
            stop_signal.clone(),
            config.rapid_gossip_sync_server_host.clone(),
        );

        let bg_join_handle = tokio::spawn(async move { p2p_background_processor.process().await });

        let peer_connector_run = peer_connector.clone();
        tokio::spawn(async move { peer_connector_run.run().await });

        let node_announcer_run = node_announcer.clone();
        tokio::spawn(async move { node_announcer_run.run().await });

        Self {
            config,
            persister,
            logger,
            network_graph,
            scorer,
            p2p_gossip,
            peer_manager,
            peer_connector,
            node_announcer,
            runtime_handle,
            stop_signal,
            join_handles: Arc::new(Mutex::new(vec![bg_join_handle])),
        }
    }

    pub fn get_router(&self) -> AnyRouter {
        match (
            self.config.remote_p2p_host.as_ref(),
            self.config.remote_p2p_token.as_ref(),
        ) {
            (Some(host), Some(token)) => {
                AnyRouter::new_remote(host.clone(), token.clone(), self.runtime_handle.clone())
            }
            _ => {
                let mut randomness: [u8; 32] = [0; 32];
                rand::thread_rng().fill_bytes(&mut randomness);
                let local_router =
                    DefaultRouter::new(self.network_graph.clone(), self.logger.clone(), randomness);
                AnyRouter::Local(local_router)
            }
        }
    }

    pub async fn stop(&self) {
        let mut join_handles = {
            let mut p2p_join_handles = self.join_handles.lock().unwrap();
            p2p_join_handles.drain(..).collect::<Vec<JoinHandle<()>>>()
        };

        for join_handle in join_handles.iter_mut() {
            let _res = join_handle.await;
        }
    }
}
