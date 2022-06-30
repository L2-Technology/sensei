// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.
pub mod channel_peer_reconnector;
pub mod peer_connector;
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
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use crate::{
    config::SenseiConfig,
    database::SenseiDatabase,
    disk::FilesystemLogger,
    node::{NetworkGraph, NetworkGraphMessageHandler, Router, RoutingPeerManager, Scorer},
    persist::{AnyKVStore, DatabaseStore, SenseiPersister},
};

use self::{channel_peer_reconnector::ChannelPeerReconnector, peer_connector::PeerConnector};

#[derive(Clone)]
pub struct SenseiP2P {
    pub config: Arc<SenseiConfig>,
    pub persister: Arc<SenseiPersister>,
    pub network_graph: Arc<NetworkGraph>,
    pub network_graph_message_handler: Arc<NetworkGraphMessageHandler>,
    pub scorer: Arc<Mutex<Scorer>>,
    pub logger: Arc<FilesystemLogger>,
    pub peer_manager: Arc<RoutingPeerManager>,
    pub channel_peer_reconnector: Arc<ChannelPeerReconnector>,
    pub peer_connector: Arc<PeerConnector>,
}

impl SenseiP2P {
    pub fn new(
        config: Arc<SenseiConfig>,
        database: Arc<SenseiDatabase>,
        logger: Arc<FilesystemLogger>,
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
        let scorer = Arc::new(Mutex::new(
            persister.read_scorer(Arc::clone(&network_graph)),
        ));
        let network_graph_message_handler = Arc::new(NetworkGraphMessageHandler::new(
            Arc::clone(&network_graph),
            None::<Arc<dyn chain::Access + Send + Sync>>,
            logger.clone(),
        ));

        let lightning_msg_handler = MessageHandler {
            chan_handler: Arc::new(ErroringMessageHandler::new()),
            route_handler: network_graph_message_handler.clone(),
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

        let peer_connector = Arc::new(PeerConnector {
            routing_peer_manager: peer_manager.clone(),
        });

        let channel_peer_reconnector = Arc::new(ChannelPeerReconnector::new(
            peer_connector.clone(),
            network_graph.clone(),
        ));

        let scorer_persister = Arc::clone(&persister);
        let scorer_persist = Arc::clone(&scorer);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600));
            loop {
                interval.tick().await;
                if scorer_persister
                    .persist_scorer(&scorer_persist.lock().unwrap())
                    .is_err()
                {
                    // Persistence errors here are non-fatal as channels will be re-scored as payments
                    // fail, but they may indicate a disk error which could be fatal elsewhere.
                    eprintln!("Warning: Failed to persist scorer, check your disk and permissions");
                }
            }
        });

        let channel_peer_reconnector_run = channel_peer_reconnector.clone();
        tokio::spawn(async move { channel_peer_reconnector_run.run().await });

        Self {
            config,
            persister,
            logger,
            network_graph,
            scorer,
            network_graph_message_handler,
            peer_manager,
            channel_peer_reconnector,
            peer_connector,
        }
    }

    pub fn get_router(&self) -> Router {
        let mut randomness: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut randomness);
        DefaultRouter::new(self.network_graph.clone(), self.logger.clone(), randomness)
    }
}
