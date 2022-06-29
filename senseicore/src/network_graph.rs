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
    ln::msgs::{self, Init, LightningError, RoutingMessageHandler},
    util::events::{MessageSendEvent, MessageSendEventsProvider},
};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::node::{NetworkGraph, NetworkGraphMessageHandler, Scorer};

#[derive(Clone)]
pub struct SenseiNetworkGraph {
    pub graph: Option<Arc<NetworkGraph>>,
    pub msg_handler: Option<Arc<NetworkGraphMessageHandler>>,
    pub scorer: Option<Arc<Mutex<Scorer>>>,
}

impl SenseiNetworkGraph {
    pub fn set_graph(&mut self, graph: Arc<NetworkGraph>) {
        self.graph = Some(graph);
    }

    pub fn get_graph(&self) -> Option<Arc<NetworkGraph>> {
        self.graph.clone()
    }

    pub fn set_msg_handler(&mut self, msg_handler: Arc<NetworkGraphMessageHandler>) {
        self.msg_handler = Some(msg_handler);
    }

    pub fn get_msg_handler(&self) -> Option<Arc<NetworkGraphMessageHandler>> {
        self.msg_handler.clone()
    }

    pub fn set_scorer(&mut self, scorer: Arc<Mutex<Scorer>>) {
        self.scorer = Some(scorer);
    }

    pub fn get_scorer(&self) -> Option<Arc<Mutex<Scorer>>> {
        self.scorer.clone()
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
