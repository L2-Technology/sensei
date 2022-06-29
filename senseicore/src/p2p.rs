// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use bitcoin::secp256k1::PublicKey;
use lightning::{
    chain,
    ln::msgs::{self, Init, LightningError, RoutingMessageHandler},
    util::events::{MessageSendEvent, MessageSendEventsProvider},
};
use lightning_invoice::utils::DefaultRouter;
use rand::RngCore;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    config::SenseiConfig,
    database::SenseiDatabase,
    disk::FilesystemLogger,
    node::{NetworkGraph, NetworkGraphMessageHandler, Router, Scorer},
    persist::{AnyKVStore, DatabaseStore, SenseiPersister},
};

#[derive(Clone)]
pub struct SenseiP2P {
    pub config: Arc<SenseiConfig>,
    pub persister: Arc<SenseiPersister>,
    pub network_graph: Arc<NetworkGraph>,
    pub network_graph_message_handler: Arc<NetworkGraphMessageHandler>,
    pub scorer: Arc<Mutex<Scorer>>,
    pub logger: Arc<FilesystemLogger>,
}

impl SenseiP2P {
    pub fn new(
        config: Arc<SenseiConfig>,
        database: Arc<SenseiDatabase>,
        logger: Arc<FilesystemLogger>,
    ) -> Self {
        let persistence_store =
            AnyKVStore::Database(DatabaseStore::new(database, "SENSEI".to_string()));

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

        Self {
            config,
            persister,
            logger,
            network_graph,
            scorer,
            network_graph_message_handler,
        }
    }

    pub fn get_router(&self) -> Router {
        let mut randomness: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut randomness);
        DefaultRouter::new(self.network_graph.clone(), self.logger.clone(), randomness)
    }
}
pub struct OptionalNetworkGraphMsgHandler {
    pub network_graph_msg_handler: Option<Arc<NetworkGraphMessageHandler>>,
}

impl MessageSendEventsProvider for OptionalNetworkGraphMsgHandler {
    fn get_and_clear_pending_msg_events(&self) -> Vec<MessageSendEvent> {
        match &self.network_graph_msg_handler {
            None => Vec::new(),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.get_and_clear_pending_msg_events()
            }
        }
    }
}

impl RoutingMessageHandler for OptionalNetworkGraphMsgHandler {
    fn handle_node_announcement(
        &self,
        _msg: &msgs::NodeAnnouncement,
    ) -> Result<bool, LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(false),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_node_announcement(_msg)
            }
        }
    }

    fn peer_connected(&self, their_node_id: &PublicKey, init: &Init) {
        match &self.network_graph_msg_handler {
            None => {}
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.peer_connected(their_node_id, init)
            }
        }
    }

    fn handle_channel_announcement(
        &self,
        _msg: &msgs::ChannelAnnouncement,
    ) -> Result<bool, LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(false),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_channel_announcement(_msg)
            }
        }
    }

    fn handle_channel_update(&self, _msg: &msgs::ChannelUpdate) -> Result<bool, LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(false),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_channel_update(_msg)
            }
        }
    }

    fn get_next_channel_announcements(
        &self,
        _starting_point: u64,
        _batch_amount: u8,
    ) -> Vec<(
        msgs::ChannelAnnouncement,
        Option<msgs::ChannelUpdate>,
        Option<msgs::ChannelUpdate>,
    )> {
        match &self.network_graph_msg_handler {
            None => Vec::new(),
            Some(network_graph_msg_handler) => network_graph_msg_handler
                .get_next_channel_announcements(_starting_point, _batch_amount),
        }
    }

    fn get_next_node_announcements(
        &self,
        _starting_point: Option<&PublicKey>,
        _batch_amount: u8,
    ) -> Vec<msgs::NodeAnnouncement> {
        match &self.network_graph_msg_handler {
            None => Vec::new(),
            Some(network_graph_msg_handler) => network_graph_msg_handler
                .get_next_node_announcements(_starting_point, _batch_amount),
        }
    }

    fn handle_reply_channel_range(
        &self,
        _their_node_id: &PublicKey,
        _msg: msgs::ReplyChannelRange,
    ) -> Result<(), LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_reply_channel_range(_their_node_id, _msg)
            }
        }
    }

    fn handle_reply_short_channel_ids_end(
        &self,
        _their_node_id: &PublicKey,
        _msg: msgs::ReplyShortChannelIdsEnd,
    ) -> Result<(), LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_reply_short_channel_ids_end(_their_node_id, _msg)
            }
        }
    }

    fn handle_query_channel_range(
        &self,
        _their_node_id: &PublicKey,
        _msg: msgs::QueryChannelRange,
    ) -> Result<(), LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_query_channel_range(_their_node_id, _msg)
            }
        }
    }

    fn handle_query_short_channel_ids(
        &self,
        _their_node_id: &PublicKey,
        _msg: msgs::QueryShortChannelIds,
    ) -> Result<(), LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_query_short_channel_ids(_their_node_id, _msg)
            }
        }
    }
}

impl Deref for OptionalNetworkGraphMsgHandler {
    type Target = OptionalNetworkGraphMsgHandler;
    fn deref(&self) -> &Self {
        self
    }
}
