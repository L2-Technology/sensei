use crate::{hex_utils, node::NetworkGraphMessageHandler};
use lightning::util::ser::Writeable;
use lightning::{
    ln::msgs::RoutingMessageHandler,
    util::events::{MessageSendEvent, MessageSendEventsProvider},
};
use std::sync::Arc;
use tokio::runtime::Handle;

use super::router::RemoteSenseiInfo;

pub enum AnyP2PGossipHandler {
    Remote(RemoteGossipMessageHandler),
    Local(NetworkGraphMessageHandler),
    None,
}

impl AnyP2PGossipHandler {
    pub fn new_remote(host: String, token: String, handle: Handle) -> Self {
        AnyP2PGossipHandler::Remote(RemoteGossipMessageHandler::new(host, token, handle))
    }
}

impl MessageSendEventsProvider for AnyP2PGossipHandler {
    fn get_and_clear_pending_msg_events(&self) -> Vec<MessageSendEvent> {
        match self {
            AnyP2PGossipHandler::Remote(remote_handler) => {
                remote_handler.get_and_clear_pending_msg_events()
            }
            AnyP2PGossipHandler::Local(local_handler) => {
                local_handler.get_and_clear_pending_msg_events()
            }
            AnyP2PGossipHandler::None => {
                vec![]
            }
        }
    }
}

impl RoutingMessageHandler for AnyP2PGossipHandler {
    fn handle_node_announcement(
        &self,
        msg: &lightning::ln::msgs::NodeAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => handler.handle_node_announcement(msg),
            AnyP2PGossipHandler::Local(handler) => handler.handle_node_announcement(msg),
            AnyP2PGossipHandler::None => {
                panic!("handle_node_announcement called without a P2P Gossip Handler")
            }
        }
    }

    fn handle_channel_announcement(
        &self,
        msg: &lightning::ln::msgs::ChannelAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => handler.handle_channel_announcement(msg),
            AnyP2PGossipHandler::Local(handler) => handler.handle_channel_announcement(msg),
            AnyP2PGossipHandler::None => {
                panic!("handle_channel_announcement called without a P2P Gossip Handler")
            }
        }
    }

    fn handle_channel_update(
        &self,
        msg: &lightning::ln::msgs::ChannelUpdate,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => handler.handle_channel_update(msg),
            AnyP2PGossipHandler::Local(handler) => handler.handle_channel_update(msg),
            AnyP2PGossipHandler::None => {
                panic!("handle_channel_update called without a P2P Gossip Handler")
            }
        }
    }

    fn get_next_channel_announcements(
        &self,
        starting_point: u64,
        batch_amount: u8,
    ) -> Vec<(
        lightning::ln::msgs::ChannelAnnouncement,
        Option<lightning::ln::msgs::ChannelUpdate>,
        Option<lightning::ln::msgs::ChannelUpdate>,
    )> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => {
                handler.get_next_channel_announcements(starting_point, batch_amount)
            }
            AnyP2PGossipHandler::Local(handler) => {
                handler.get_next_channel_announcements(starting_point, batch_amount)
            }
            AnyP2PGossipHandler::None => {
                panic!("get_next_channel_announcements called without a P2P Gossip Handler")
            }
        }
    }

    fn get_next_node_announcements(
        &self,
        starting_point: Option<&bitcoin::secp256k1::PublicKey>,
        batch_amount: u8,
    ) -> Vec<lightning::ln::msgs::NodeAnnouncement> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => {
                handler.get_next_node_announcements(starting_point, batch_amount)
            }
            AnyP2PGossipHandler::Local(handler) => {
                handler.get_next_node_announcements(starting_point, batch_amount)
            }
            AnyP2PGossipHandler::None => {
                panic!("get_next_node_announcements called without a P2P Gossip Handler")
            }
        }
    }

    fn peer_connected(
        &self,
        their_node_id: &bitcoin::secp256k1::PublicKey,
        init: &lightning::ln::msgs::Init,
    ) {
        match self {
            AnyP2PGossipHandler::Remote(handler) => handler.peer_connected(their_node_id, init),
            AnyP2PGossipHandler::Local(handler) => handler.peer_connected(their_node_id, init),
            AnyP2PGossipHandler::None => {
                panic!("peer_connected called without a P2P Gossip Handler")
            }
        }
    }

    fn handle_reply_channel_range(
        &self,
        their_node_id: &bitcoin::secp256k1::PublicKey,
        msg: lightning::ln::msgs::ReplyChannelRange,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => {
                handler.handle_reply_channel_range(their_node_id, msg)
            }
            AnyP2PGossipHandler::Local(handler) => {
                handler.handle_reply_channel_range(their_node_id, msg)
            }
            AnyP2PGossipHandler::None => {
                panic!("handle_reply_channel_range called without a P2P Gossip Handler")
            }
        }
    }

    fn handle_reply_short_channel_ids_end(
        &self,
        their_node_id: &bitcoin::secp256k1::PublicKey,
        msg: lightning::ln::msgs::ReplyShortChannelIdsEnd,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => {
                handler.handle_reply_short_channel_ids_end(their_node_id, msg)
            }
            AnyP2PGossipHandler::Local(handler) => {
                handler.handle_reply_short_channel_ids_end(their_node_id, msg)
            }
            AnyP2PGossipHandler::None => {
                panic!("handle_reply_short_channel_ids_end called without a P2P Gossip Handler")
            }
        }
    }

    fn handle_query_channel_range(
        &self,
        their_node_id: &bitcoin::secp256k1::PublicKey,
        msg: lightning::ln::msgs::QueryChannelRange,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => {
                handler.handle_query_channel_range(their_node_id, msg)
            }
            AnyP2PGossipHandler::Local(handler) => {
                handler.handle_query_channel_range(their_node_id, msg)
            }
            AnyP2PGossipHandler::None => {
                panic!("handle_query_channel_range called without a P2P Gossip Handler")
            }
        }
    }

    fn handle_query_short_channel_ids(
        &self,
        their_node_id: &bitcoin::secp256k1::PublicKey,
        msg: lightning::ln::msgs::QueryShortChannelIds,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        match self {
            AnyP2PGossipHandler::Remote(handler) => {
                handler.handle_query_short_channel_ids(their_node_id, msg)
            }
            AnyP2PGossipHandler::Local(handler) => {
                handler.handle_query_short_channel_ids(their_node_id, msg)
            }
            AnyP2PGossipHandler::None => {
                panic!("handle_query_short_channel_ids called without a P2P Gossip Handler")
            }
        }
    }
}

#[derive(Clone)]
pub struct RemoteGossipMessageHandler {
    remote_sensei: RemoteSenseiInfo,
    tokio_handle: Handle,
}

impl RemoteGossipMessageHandler {
    pub fn new(host: String, token: String, tokio_handle: Handle) -> Self {
        Self {
            remote_sensei: RemoteSenseiInfo { host, token },
            tokio_handle,
        }
    }

    fn node_announcement_path(&self) -> String {
        format!(
            "{}/v1/ldk/network/gossip/node-announcement",
            self.remote_sensei.host
        )
    }

    fn channel_announcement_path(&self) -> String {
        format!(
            "{}/v1/ldk/network/gossip/channel-announcement",
            self.remote_sensei.host
        )
    }

    fn channel_update_path(&self) -> String {
        format!(
            "{}/v1/ldk/network/gossip/channel-update",
            self.remote_sensei.host
        )
    }

    async fn handle_node_announcement_async(
        &self,
        msg: &lightning::ln::msgs::NodeAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        let client = reqwest::Client::new();
        let _res = client
            .post(self.node_announcement_path())
            .header("token", self.remote_sensei.token.clone())
            .json(&serde_json::json!({
              "msg_hex": hex_utils::hex_str(&msg.encode()),
            }))
            .send()
            .await;
        Ok(true)
    }

    async fn handle_channel_announcement_async(
        &self,
        msg: &lightning::ln::msgs::ChannelAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        let client = reqwest::Client::new();
        let _res = client
            .post(self.channel_announcement_path())
            .header("token", self.remote_sensei.token.clone())
            .json(&serde_json::json!({
              "msg_hex": hex_utils::hex_str(&msg.encode()),
            }))
            .send()
            .await;

        Ok(true)
    }

    async fn handle_channel_update_async(
        &self,
        msg: &lightning::ln::msgs::ChannelUpdate,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        let client = reqwest::Client::new();
        let _res = client
            .post(self.channel_update_path())
            .header("token", self.remote_sensei.token.clone())
            .json(&serde_json::json!({
              "msg_hex": hex_utils::hex_str(&msg.encode()),
            }))
            .send()
            .await;
        Ok(true)
    }
}
impl MessageSendEventsProvider for RemoteGossipMessageHandler {
    fn get_and_clear_pending_msg_events(&self) -> Vec<MessageSendEvent> {
        Vec::new()
    }
}

impl RoutingMessageHandler for RemoteGossipMessageHandler {
    fn handle_node_announcement(
        &self,
        msg: &lightning::ln::msgs::NodeAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        tokio::task::block_in_place(move || {
            self.tokio_handle
                .clone()
                .block_on(async move { self.handle_node_announcement_async(msg).await })
        })
    }

    fn handle_channel_announcement(
        &self,
        msg: &lightning::ln::msgs::ChannelAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        tokio::task::block_in_place(move || {
            self.tokio_handle
                .clone()
                .block_on(async move { self.handle_channel_announcement_async(msg).await })
        })
    }

    fn handle_channel_update(
        &self,
        msg: &lightning::ln::msgs::ChannelUpdate,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        tokio::task::block_in_place(move || {
            self.tokio_handle
                .clone()
                .block_on(async move { self.handle_channel_update_async(msg).await })
        })
    }

    fn get_next_channel_announcements(
        &self,
        _starting_point: u64,
        _batch_amount: u8,
    ) -> Vec<(
        lightning::ln::msgs::ChannelAnnouncement,
        Option<lightning::ln::msgs::ChannelUpdate>,
        Option<lightning::ln::msgs::ChannelUpdate>,
    )> {
        Vec::new()
    }

    fn get_next_node_announcements(
        &self,
        _starting_point: Option<&bitcoin::secp256k1::PublicKey>,
        _batch_amount: u8,
    ) -> Vec<lightning::ln::msgs::NodeAnnouncement> {
        Vec::new()
    }

    fn peer_connected(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _init: &lightning::ln::msgs::Init,
    ) {
    }

    fn handle_reply_channel_range(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::ReplyChannelRange,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_reply_short_channel_ids_end(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::ReplyShortChannelIdsEnd,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_query_channel_range(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::QueryChannelRange,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_query_short_channel_ids(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::QueryShortChannelIds,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }
}

pub struct BubbleGossipRouteHandler {
    pub target: Arc<AnyP2PGossipHandler>,
}

impl MessageSendEventsProvider for BubbleGossipRouteHandler {
    fn get_and_clear_pending_msg_events(&self) -> Vec<MessageSendEvent> {
        Vec::new()
    }
}

impl RoutingMessageHandler for BubbleGossipRouteHandler {
    fn handle_node_announcement(
        &self,
        msg: &lightning::ln::msgs::NodeAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        self.target.handle_node_announcement(msg)
    }

    fn handle_channel_announcement(
        &self,
        msg: &lightning::ln::msgs::ChannelAnnouncement,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        self.target.handle_channel_announcement(msg)
    }

    fn handle_channel_update(
        &self,
        msg: &lightning::ln::msgs::ChannelUpdate,
    ) -> Result<bool, lightning::ln::msgs::LightningError> {
        self.target.handle_channel_update(msg)
    }

    fn get_next_channel_announcements(
        &self,
        _starting_point: u64,
        _batch_amount: u8,
    ) -> Vec<(
        lightning::ln::msgs::ChannelAnnouncement,
        Option<lightning::ln::msgs::ChannelUpdate>,
        Option<lightning::ln::msgs::ChannelUpdate>,
    )> {
        Vec::new()
    }

    fn get_next_node_announcements(
        &self,
        _starting_point: Option<&bitcoin::secp256k1::PublicKey>,
        _batch_amount: u8,
    ) -> Vec<lightning::ln::msgs::NodeAnnouncement> {
        Vec::new()
    }

    fn peer_connected(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _init: &lightning::ln::msgs::Init,
    ) {
    }

    fn handle_reply_channel_range(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::ReplyChannelRange,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_reply_short_channel_ids_end(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::ReplyShortChannelIdsEnd,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_query_channel_range(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::QueryChannelRange,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }

    fn handle_query_short_channel_ids(
        &self,
        _their_node_id: &bitcoin::secp256k1::PublicKey,
        _msg: lightning::ln::msgs::QueryShortChannelIds,
    ) -> Result<(), lightning::ln::msgs::LightningError> {
        Ok(())
    }
}
