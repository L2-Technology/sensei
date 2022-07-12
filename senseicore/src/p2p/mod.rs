// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.
pub mod background_processor;
pub mod node_announcer;
pub mod node_info;
pub mod peer_connector;
pub mod router;
pub mod utils;
pub mod bubble_gossip_route_handler;

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
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::{
    config::SenseiConfig,
    database::SenseiDatabase,
    disk::FilesystemLogger,
    node::{NetworkGraph, NetworkGraphMessageHandler, RoutingPeerManager},
    persist::{AnyKVStore, DatabaseStore, SenseiPersister},
};

use self::{
    background_processor::BackgroundProcessor,
    node_announcer::NodeAnnouncer,
    node_info::NodeInfoLookup,
    peer_connector::PeerConnector,
    router::{AnyRouter, AnyScorer},
    utils::parse_peer_info,
};

#[derive(Clone)]
pub struct SenseiP2P {
    pub config: Arc<SenseiConfig>,
    pub persister: Arc<SenseiPersister>,
    pub network_graph: Arc<NetworkGraph>,
    pub p2p_gossip: Arc<NetworkGraphMessageHandler>,
    pub scorer: Arc<Mutex<AnyScorer>>,
    pub logger: Arc<FilesystemLogger>,
    pub peer_manager: Arc<RoutingPeerManager>,
    pub peer_connector: Arc<PeerConnector>,
    pub node_announcer: Arc<NodeAnnouncer>,
    pub runtime_handle: tokio::runtime::Handle,
}

impl SenseiP2P {
    pub async fn new(
        config: Arc<SenseiConfig>,
        database: Arc<SenseiDatabase>,
        logger: Arc<FilesystemLogger>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        let p2p_node_id = "SENSEI".to_string();

        let persistence_store =
            AnyKVStore::Database(DatabaseStore::new(database.clone(), p2p_node_id.clone()));

        let persister = Arc::new(SenseiPersister::new(
            persistence_store,
            config.network,
            logger.clone(),
        ));
        let network_graph = Arc::new(persister.read_network_graph());

        let scorer = match (
            config.remote_p2p_host.as_ref(),
            config.remote_p2p_token.as_ref(),
        ) {
            (Some(host), Some(token)) => Arc::new(Mutex::new(AnyScorer::new_remote(
                host.clone(),
                token.clone(),
                runtime_handle.clone(),
            ))),
            _ => Arc::new(Mutex::new(AnyScorer::Local(
                persister.read_scorer(Arc::clone(&network_graph)),
            ))),
        };

        let p2p_gossip = Arc::new(NetworkGraphMessageHandler::new(
            Arc::clone(&network_graph),
            None::<Arc<dyn chain::Access + Send + Sync>>,
            logger.clone(),
        ));

        let lightning_msg_handler = MessageHandler {
            chan_handler: Arc::new(ErroringMessageHandler::new()),
            route_handler: p2p_gossip.clone(),
        };

        let mut seed: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut seed);

        match database.get_seed_sync(p2p_node_id.clone()).unwrap() {
            Some(seed_vec) => {
                seed.copy_from_slice(seed_vec.as_slice());
            }
            None => {
                let _res = database.set_seed_sync(p2p_node_id, seed.to_vec());
            }
        }

        let cur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let keys_manager = Arc::new(KeysManager::new(&seed, cur.as_secs(), cur.subsec_nanos()));

        let mut ephemeral_bytes = [0; 32];
        rand::thread_rng().fill_bytes(&mut ephemeral_bytes);

        let peer_manager = Arc::new(RoutingPeerManager::new(
            lightning_msg_handler,
            keys_manager.get_node_secret(Recipient::Node).unwrap(),
            &ephemeral_bytes,
            logger.clone(),
            Arc::new(IgnoringMessageHandler {}),
        ));

        let node_info_lookup = match (
            config.remote_p2p_host.as_ref(),
            config.remote_p2p_token.as_ref(),
        ) {
            (Some(host), Some(token)) => Arc::new(NodeInfoLookup::new_remote(
                host.clone(),
                token.clone(),
                runtime_handle.clone(),
            )),
            _ => Arc::new(NodeInfoLookup::Local(network_graph.clone())),
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
        );
        tokio::spawn(async move { p2p_background_processor.process().await });

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
}
