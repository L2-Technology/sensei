// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::database::node::Payment;
use crate::node::LightningNode;
use bdk::TransactionDetails;
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

use crate::hex_utils;

use lightning::ln::channelmanager::ChannelDetails;
use serde::Serialize;

use super::{PaginationRequest, PaginationResponse, PaymentsFilter};

#[derive(Serialize)]
pub struct Peer {
    pub node_pubkey: String,
}

#[derive(Serialize)]
pub struct NodeInfo {
    pub node_pubkey: String,
    pub num_channels: u32,
    pub num_usable_channels: u32,
    pub num_peers: u32,
    pub local_balance_msat: u64,
}

// #[derive(Serialize)]
// pub struct Payment {
//     pub hash: String,
//     pub preimage: Option<String>,
//     pub secret: Option<String>,
//     pub status: HTLCStatus,
//     pub amt_msat: Option<u64>,
//     pub htlc_direction: u8,
// }

// impl From<database::node::Payment> for Payment {
//     fn from(payment: database::node::Payment) -> Self {
//         let status = match payment.status.as_str() {
//             "pending" => HTLCStatus::Pending,
//             "succedded" => HTLCStatus::Succeeded,
//             "failed" => HTLCStatus::Failed,
//             _ => HTLCStatus::Unknown
//         };

//         Self {
//             hash: hex_utils::hex_str(&payment.payment_hash),
//             preimage: payment.preimage.map(|preimage| hex_utils::hex_str(&preimage)),
//             secret: payment.secret.map(|secret| hex_utils::hex_str(&secret)),
//             status,
//             amt_msat: payment.amt_msat,
//             htlc_direction: payment.direction
//         }
//     }
// }

#[derive(Serialize, Clone, Debug)]
pub struct Channel {
    pub channel_id: String,
    pub funding_txid: Option<String>,
    pub funding_tx_index: Option<u32>,
    pub short_channel_id: Option<u64>,
    pub channel_value_satoshis: u64,
    pub balance_msat: u64,
    pub unspendable_punishment_reserve: Option<u64>,
    pub user_channel_id: u64,
    pub outbound_capacity_msat: u64,
    pub inbound_capacity_msat: u64,
    pub confirmations_required: Option<u32>,
    pub force_close_spend_delay: Option<u32>,
    pub is_outbound: bool,
    pub is_funding_locked: bool,
    pub is_usable: bool,
    pub is_public: bool,
    pub counterparty_pubkey: String,
    pub alias: Option<String>,
}

impl From<ChannelDetails> for Channel {
    fn from(channel_detail: ChannelDetails) -> Self {
        Self {
            channel_id: hex_utils::hex_str(&channel_detail.channel_id),
            funding_txid: channel_detail.funding_txo.map(|txo| txo.txid.to_string()),
            funding_tx_index: channel_detail.funding_txo.map(|txo| txo.index as u32),
            short_channel_id: channel_detail.short_channel_id,
            channel_value_satoshis: channel_detail.channel_value_satoshis,
            balance_msat: channel_detail.balance_msat,
            unspendable_punishment_reserve: channel_detail.unspendable_punishment_reserve,
            user_channel_id: channel_detail.user_channel_id,
            outbound_capacity_msat: channel_detail.outbound_capacity_msat,
            inbound_capacity_msat: channel_detail.inbound_capacity_msat,
            confirmations_required: channel_detail.confirmations_required,
            force_close_spend_delay: channel_detail
                .force_close_spend_delay
                .map(|delay| delay as u32),
            is_outbound: channel_detail.is_outbound,
            is_funding_locked: channel_detail.is_funding_locked,
            is_usable: channel_detail.is_usable,
            is_public: channel_detail.is_public,
            counterparty_pubkey: channel_detail.counterparty.node_id.to_string(),
            alias: None,
        }
    }
}

pub enum NodeRequest {
    StartNode {
        passphrase: String,
    },
    StopNode {},
    GetUnusedAddress {},
    GetBalance {},
    OpenChannel {
        node_connection_string: String,
        amt_satoshis: u64,
        public: bool,
    },
    SendPayment {
        invoice: String,
    },
    Keysend {
        dest_pubkey: String,
        amt_msat: u64,
    },
    GetInvoice {
        amt_msat: u64,
        description: String,
    },
    LabelPayment {
        label: String,
        payment_hash: String,
    },
    DeletePayment {
        payment_hash: String,
    },
    ConnectPeer {
        node_connection_string: String,
    },
    ListChannels {
        pagination: PaginationRequest,
    },
    ListPayments {
        pagination: PaginationRequest,
        filter: PaymentsFilter,
    },
    ListTransactions {
        pagination: PaginationRequest,
    },
    CloseChannel {
        channel_id: String,
        force: bool,
    },
    NodeInfo {},
    ListPeers {},
    SignMessage {
        message: String,
    },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum NodeResponse {
    StartNode {},
    StopNode {},
    GetUnusedAddress {
        address: String,
    },
    GetBalance {
        balance_satoshis: u64,
    },
    OpenChannel {},
    SendPayment {},
    Keysend {},
    GetInvoice {
        invoice: String,
    },
    LabelPayment {},
    DeletePayment {},
    ConnectPeer {},
    ListChannels {
        channels: Vec<Channel>,
        pagination: PaginationResponse,
    },
    ListPayments {
        payments: Vec<Payment>,
        pagination: PaginationResponse,
    },
    ListTransactions {
        transactions: Vec<TransactionDetails>,
        pagination: PaginationResponse,
    },
    CloseChannel {},
    NodeInfo {
        node_info: NodeInfo,
    },
    ListPeers {
        peers: Vec<Peer>,
    },
    SignMessage {
        signature: String,
    },
    Error(NodeRequestError),
}

#[derive(Serialize, Debug)]
pub enum NodeRequestError {
    Sensei(String),
    BdkLdk(String),
    Bdk(String),
    Io(String),
}

impl From<bdk_ldk::Error> for NodeRequestError {
    fn from(e: bdk_ldk::Error) -> Self {
        match e {
            bdk_ldk::Error::Bdk(e) => Self::BdkLdk(e.to_string()),
        }
    }
}

impl From<bdk::Error> for NodeRequestError {
    fn from(e: bdk::Error) -> Self {
        Self::Bdk(e.to_string())
    }
}

impl From<std::io::Error> for NodeRequestError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<crate::error::Error> for NodeRequestError {
    fn from(e: crate::error::Error) -> Self {
        Self::Sensei(e.to_string())
    }
}

pub type NodeRequestFuture<R, E> = Pin<Box<dyn Future<Output = Result<R, E>>>>;

impl Service<NodeRequest> for LightningNode {
    type Response = NodeResponse;
    type Error = NodeRequestError;
    type Future = NodeRequestFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: NodeRequest) -> Self::Future {
        let this = self.clone();

        let fut = async move { this.call(request).await };

        Box::pin(fut)
    }
}
