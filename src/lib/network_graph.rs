use bitcoin::secp256k1::PublicKey;
use lightning::{util::events::{MessageSendEventsProvider, MessageSendEvent}, ln::msgs::{RoutingMessageHandler, self, LightningError}};
use std::{ops::Deref, sync::Arc};

use crate::node::NetworkGraphMessageHandler;

pub struct OptionalNetworkGraphMsgHandler {
    pub network_graph_msg_handler: Option<Arc<NetworkGraphMessageHandler>>    
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
	fn handle_node_announcement(&self, _msg: &msgs::NodeAnnouncement) -> Result<bool, LightningError> {
        match &self.network_graph_msg_handler {
            None => Ok(false),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_node_announcement(_msg)
            }
        }
    }

	fn handle_channel_announcement(&self, _msg: &msgs::ChannelAnnouncement) -> Result<bool, LightningError> { 
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

	fn get_next_channel_announcements(&self, _starting_point: u64, _batch_amount: u8) ->
		Vec<(msgs::ChannelAnnouncement, Option<msgs::ChannelUpdate>, Option<msgs::ChannelUpdate>)> { 
            match &self.network_graph_msg_handler {
                None => Vec::new(),
                Some(network_graph_msg_handler) => {
                    network_graph_msg_handler.get_next_channel_announcements(_starting_point, _batch_amount)
                }
            }
    }

	fn get_next_node_announcements(&self, _starting_point: Option<&PublicKey>, _batch_amount: u8) -> Vec<msgs::NodeAnnouncement> { 
        match &self.network_graph_msg_handler {
            None => Vec::new(),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.get_next_node_announcements(_starting_point, _batch_amount)
            }
        }
    }

	fn sync_routing_table(&self, _their_node_id: &PublicKey, _init: &msgs::Init) {
        match &self.network_graph_msg_handler {
            None => (),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.sync_routing_table(_their_node_id, _init)
            }
        }
    }

	fn handle_reply_channel_range(&self, _their_node_id: &PublicKey, _msg: msgs::ReplyChannelRange) -> Result<(), LightningError> { 
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_reply_channel_range(_their_node_id, _msg)
            }
        }
    }

	fn handle_reply_short_channel_ids_end(&self, _their_node_id: &PublicKey, _msg: msgs::ReplyShortChannelIdsEnd) -> Result<(), LightningError> { 
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_reply_short_channel_ids_end(_their_node_id, _msg)
            }
        } 
    }
	
    fn handle_query_channel_range(&self, _their_node_id: &PublicKey, _msg: msgs::QueryChannelRange) -> Result<(), LightningError> { 
        match &self.network_graph_msg_handler {
            None => Ok(()),
            Some(network_graph_msg_handler) => {
                network_graph_msg_handler.handle_query_channel_range(_their_node_id, _msg)
            }
        } 
    }
	
    fn handle_query_short_channel_ids(&self, _their_node_id: &PublicKey, _msg: msgs::QueryShortChannelIds) -> Result<(), LightningError> { 
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
	fn deref(&self) -> &Self { self }
}