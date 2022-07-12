use lightning::{ln::msgs::RoutingMessageHandler, util::events::{MessageSendEvent, MessageSendEventsProvider}};

use crate::node::NetworkGraphMessageHandler;
use std::sync::Arc;

pub struct BubbleGossipRouteHandler {
  pub target: Arc<NetworkGraphMessageHandler>
}

impl MessageSendEventsProvider for BubbleGossipRouteHandler {
  fn get_and_clear_pending_msg_events(&self) -> Vec<MessageSendEvent> { Vec::new() }
}

impl RoutingMessageHandler for BubbleGossipRouteHandler {
    fn handle_node_announcement(&self, msg: &lightning::ln::msgs::NodeAnnouncement) -> Result<bool, lightning::ln::msgs::LightningError> {
        self.target.handle_node_announcement(msg)
    }

    fn handle_channel_announcement(&self, msg: &lightning::ln::msgs::ChannelAnnouncement) -> Result<bool, lightning::ln::msgs::LightningError> {
        self.target.handle_channel_announcement(msg)
    }

    fn handle_channel_update(&self, msg: &lightning::ln::msgs::ChannelUpdate) -> Result<bool, lightning::ln::msgs::LightningError> {
        self.target.handle_channel_update(msg)
    }

    fn get_next_channel_announcements(&self, _starting_point: u64, _batch_amount: u8) -> Vec<(lightning::ln::msgs::ChannelAnnouncement, Option<lightning::ln::msgs::ChannelUpdate>, Option<lightning::ln::msgs::ChannelUpdate>)> {
      Vec::new()
    }

    fn get_next_node_announcements(&self, _starting_point: Option<&bitcoin::secp256k1::PublicKey>, _batch_amount: u8) -> Vec<lightning::ln::msgs::NodeAnnouncement> {
      Vec::new()
    }

    fn peer_connected(&self, _their_node_id: &bitcoin::secp256k1::PublicKey, _init: &lightning::ln::msgs::Init) {}

    fn handle_reply_channel_range(&self, _their_node_id: &bitcoin::secp256k1::PublicKey, _msg: lightning::ln::msgs::ReplyChannelRange) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_reply_short_channel_ids_end(&self, _their_node_id: &bitcoin::secp256k1::PublicKey, _msg: lightning::ln::msgs::ReplyShortChannelIdsEnd) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_query_channel_range(&self, _their_node_id: &bitcoin::secp256k1::PublicKey, _msg: lightning::ln::msgs::QueryChannelRange) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_query_short_channel_ids(&self, _their_node_id: &bitcoin::secp256k1::PublicKey, _msg: lightning::ln::msgs::QueryShortChannelIds) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }
}