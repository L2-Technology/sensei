// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use super::sensei::{
    self, AddKnownPeerRequest, AddKnownPeerResponse, Channel as ChannelMessage,
    CreatePhantomInvoiceRequest, CreatePhantomInvoiceResponse, DeletePaymentRequest,
    DeletePaymentResponse, GetPhantomRouteHintsRequest, GetPhantomRouteHintsResponse,
    Info as InfoMessage, KnownPeer, LabelPaymentRequest, LabelPaymentResponse,
    ListKnownPeersRequest, ListKnownPeersResponse, ListPhantomPaymentsRequest,
    ListPhantomPaymentsResponse, NetworkGraphInfoRequest, NetworkGraphInfoResponse,
    OpenChannelRequest as GrpcOpenChannelRequest, OpenChannelsRequest, OpenChannelsResponse,
    PaginationRequest, PaginationResponse, Payment as PaymentMessage, PaymentsFilter,
    Peer as PeerMessage, RemoveKnownPeerRequest, RemoveKnownPeerResponse, StartNodeRequest,
    StartNodeResponse, StopNodeRequest, StopNodeResponse, Utxo as UtxoMessage,
};

use super::sensei::{
    CloseChannelRequest, CloseChannelResponse, ConnectPeerRequest, ConnectPeerResponse,
    CreateInvoiceRequest, CreateInvoiceResponse, DecodeInvoiceRequest, DecodeInvoiceResponse,
    GetBalanceRequest, GetBalanceResponse, GetUnusedAddressRequest, GetUnusedAddressResponse,
    InfoRequest, InfoResponse, KeysendRequest, KeysendResponse, ListChannelsRequest,
    ListChannelsResponse, ListPaymentsRequest, ListPaymentsResponse, ListPeersRequest,
    ListPeersResponse, ListUnspentRequest, ListUnspentResponse, PayInvoiceRequest,
    PayInvoiceResponse, SignMessageRequest, SignMessageResponse, VerifyMessageRequest,
    VerifyMessageResponse,
};

use senseicore::services::node::OpenChannelRequest;
use senseicore::services::{
    self,
    node::{Channel, NodeInfo, NodeRequest, NodeResponse, Peer, Utxo},
};

impl From<PaymentsFilter> for services::PaymentsFilter {
    fn from(filter: PaymentsFilter) -> Self {
        Self {
            origin: filter.origin,
            status: filter.status,
        }
    }
}

impl From<PaginationRequest> for services::PaginationRequest {
    fn from(pagination: PaginationRequest) -> Self {
        Self {
            take: pagination.take,
            page: pagination.page,
            query: pagination.query,
        }
    }
}

impl From<services::PaginationResponse> for PaginationResponse {
    fn from(pagination: services::PaginationResponse) -> Self {
        Self {
            has_more: pagination.has_more,
            total: pagination.total,
        }
    }
}

impl From<Channel> for ChannelMessage {
    fn from(channel: Channel) -> Self {
        Self {
            channel_id: channel.channel_id,
            funding_txid: channel.funding_txid,
            funding_tx_index: channel.funding_tx_index,
            short_channel_id: channel.short_channel_id,
            channel_value_satoshis: channel.channel_value_satoshis,
            balance_msat: channel.balance_msat,
            unspendable_punishment_reserve: channel.unspendable_punishment_reserve,
            user_channel_id: channel.user_channel_id,
            outbound_capacity_msat: channel.outbound_capacity_msat,
            inbound_capacity_msat: channel.inbound_capacity_msat,
            confirmations_required: channel.confirmations_required,
            force_close_spend_delay: channel.force_close_spend_delay,
            is_outbound: channel.is_outbound,
            is_channel_ready: channel.is_channel_ready,
            is_usable: channel.is_usable,
            is_public: channel.is_public,
            counterparty_pubkey: channel.counterparty_pubkey,
            alias: channel.alias,
        }
    }
}

impl From<entity::payment::Model> for PaymentMessage {
    fn from(payment: entity::payment::Model) -> Self {
        Self {
            node_id: payment.node_id,
            hash: payment.payment_hash,
            preimage: payment.preimage,
            secret: payment.secret,
            status: payment.status,
            amt_msat: payment.amt_msat,
            fee_paid_msat: payment.fee_paid_msat,
            origin: payment.origin,
            label: payment.label,
            invoice: payment.invoice,
            created_by_node_id: payment.created_by_node_id,
            received_by_node_id: payment.received_by_node_id,
        }
    }
}

impl From<NodeInfo> for InfoMessage {
    fn from(info: NodeInfo) -> Self {
        Self {
            version: info.version,
            node_pubkey: info.node_pubkey,
            num_channels: info.num_channels,
            num_usable_channels: info.num_usable_channels,
            num_peers: info.num_peers,
        }
    }
}

impl From<Peer> for PeerMessage {
    fn from(peer: Peer) -> Self {
        Self {
            node_pubkey: peer.node_pubkey,
        }
    }
}

impl From<Utxo> for UtxoMessage {
    fn from(utxo: Utxo) -> Self {
        Self {
            amount_sat: utxo.amount_sat,
            spk: utxo.spk,
            txid: utxo.txid,
            output_index: utxo.output_index,
        }
    }
}

impl TryFrom<NodeResponse> for StartNodeResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::StartNode {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<StartNodeRequest> for NodeRequest {
    fn from(req: StartNodeRequest) -> Self {
        NodeRequest::StartNode {
            passphrase: req.passphrase,
        }
    }
}

impl From<StopNodeRequest> for NodeRequest {
    fn from(_req: StopNodeRequest) -> Self {
        NodeRequest::StopNode {}
    }
}

impl TryFrom<NodeResponse> for StopNodeResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::StopNode {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<GetUnusedAddressRequest> for NodeRequest {
    fn from(_req: GetUnusedAddressRequest) -> Self {
        NodeRequest::GetUnusedAddress {}
    }
}

impl TryFrom<NodeResponse> for GetUnusedAddressResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::GetUnusedAddress { address } => Ok(Self { address }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<GetBalanceRequest> for NodeRequest {
    fn from(_req: GetBalanceRequest) -> Self {
        NodeRequest::GetBalance {}
    }
}

impl TryFrom<NodeResponse> for GetBalanceResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::GetBalance {
                onchain_balance_sats,
                channel_balance_msats,
                channel_outbound_capacity_msats,
                channel_inbound_capacity_msats,
                usable_channel_outbound_capacity_msats,
                usable_channel_inbound_capacity_msats,
            } => Ok(Self {
                onchain_balance_sats,
                channel_balance_msats,
                channel_outbound_capacity_msats,
                channel_inbound_capacity_msats,
                usable_channel_outbound_capacity_msats,
                usable_channel_inbound_capacity_msats,
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<OpenChannelsRequest> for NodeRequest {
    fn from(req: OpenChannelsRequest) -> Self {
        NodeRequest::OpenChannels {
            requests: req
                .requests
                .into_iter()
                .map(|request| OpenChannelRequest {
                    counterparty_pubkey: request.counterparty_pubkey,
                    amount_sats: request.amount_sats,
                    public: request.public,
                    scid_alias: request.scid_alias,
                    custom_id: request.custom_id,
                    push_amount_msats: request.push_amount_msats,
                    counterparty_host_port: request.counterparty_host_port,
                    forwarding_fee_proportional_millionths: request
                        .forwarding_fee_proportional_millionths,
                    forwarding_fee_base_msat: request.forwarding_fee_base_msat,
                    cltv_expiry_delta: request
                        .cltv_expiry_delta
                        .map(|cltv_delta| cltv_delta as u16),
                    max_dust_htlc_exposure_msat: request.max_dust_htlc_exposure_msat,
                    force_close_avoidance_max_fee_satoshis: request
                        .force_close_avoidance_max_fee_satoshis,
                })
                .collect::<Vec<_>>(),
        }
    }
}

impl TryFrom<NodeResponse> for OpenChannelsResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::OpenChannels { requests, results } => Ok(Self {
                requests: requests
                    .into_iter()
                    .map(|request| GrpcOpenChannelRequest {
                        counterparty_pubkey: request.counterparty_pubkey,
                        amount_sats: request.amount_sats,
                        public: request.public,
                        scid_alias: request.scid_alias,
                        custom_id: request.custom_id,
                        push_amount_msats: request.push_amount_msats,
                        counterparty_host_port: request.counterparty_host_port,
                        forwarding_fee_proportional_millionths: request
                            .forwarding_fee_proportional_millionths,
                        forwarding_fee_base_msat: request.forwarding_fee_base_msat,
                        cltv_expiry_delta: request
                            .cltv_expiry_delta
                            .map(|cltv_delta| cltv_delta.try_into().unwrap()),
                        max_dust_htlc_exposure_msat: request.max_dust_htlc_exposure_msat,
                        force_close_avoidance_max_fee_satoshis: request
                            .force_close_avoidance_max_fee_satoshis,
                    })
                    .collect::<Vec<_>>(),
                results: results
                    .into_iter()
                    .map(|result| sensei::OpenChannelResult {
                        error: result.error,
                        error_message: result.error_message,
                        channel_id: result.channel_id,
                    })
                    .collect::<Vec<_>>(),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<PayInvoiceRequest> for NodeRequest {
    fn from(req: PayInvoiceRequest) -> Self {
        NodeRequest::SendPayment {
            invoice: req.invoice,
        }
    }
}

impl TryFrom<NodeResponse> for PayInvoiceResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::SendPayment {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<DecodeInvoiceRequest> for NodeRequest {
    fn from(req: DecodeInvoiceRequest) -> Self {
        NodeRequest::DecodeInvoice {
            invoice: req.invoice,
        }
    }
}

impl TryFrom<NodeResponse> for DecodeInvoiceResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::DecodeInvoice { invoice } => Ok(Self {
                invoice: Some(invoice.into()),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<KeysendRequest> for NodeRequest {
    fn from(req: KeysendRequest) -> Self {
        NodeRequest::Keysend {
            dest_pubkey: req.dest_pubkey,
            amt_msat: req.amt_msat,
        }
    }
}

impl TryFrom<NodeResponse> for KeysendResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::Keysend {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<CreateInvoiceRequest> for NodeRequest {
    fn from(req: CreateInvoiceRequest) -> Self {
        NodeRequest::GetInvoice {
            amt_msat: req.amt_msat,
            description: req.description,
        }
    }
}

impl TryFrom<NodeResponse> for CreateInvoiceResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::GetInvoice { invoice } => Ok(Self { invoice }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<CreatePhantomInvoiceRequest> for NodeRequest {
    fn from(req: CreatePhantomInvoiceRequest) -> Self {
        NodeRequest::GetPhantomInvoice {
            amt_msat: req.amt_msat,
            description: req.description,
            phantom_route_hints_hex: req.phantom_route_hints_hex,
        }
    }
}

impl TryFrom<NodeResponse> for CreatePhantomInvoiceResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::GetPhantomInvoice { invoice } => Ok(Self { invoice }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<GetPhantomRouteHintsRequest> for NodeRequest {
    fn from(_req: GetPhantomRouteHintsRequest) -> Self {
        NodeRequest::GetPhantomRouteHints {}
    }
}

impl TryFrom<NodeResponse> for GetPhantomRouteHintsResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::GetPhantomRouteHints {
                phantom_route_hints_hex,
            } => Ok(Self {
                phantom_route_hints_hex,
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<LabelPaymentRequest> for NodeRequest {
    fn from(req: LabelPaymentRequest) -> Self {
        NodeRequest::LabelPayment {
            label: req.label,
            payment_hash: req.payment_hash,
        }
    }
}

impl TryFrom<NodeResponse> for LabelPaymentResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::LabelPayment {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<DeletePaymentRequest> for NodeRequest {
    fn from(req: DeletePaymentRequest) -> Self {
        NodeRequest::DeletePayment {
            payment_hash: req.payment_hash,
        }
    }
}

impl TryFrom<NodeResponse> for DeletePaymentResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::DeletePayment {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ConnectPeerRequest> for NodeRequest {
    fn from(req: ConnectPeerRequest) -> Self {
        NodeRequest::ConnectPeer {
            node_connection_string: req.node_connection_string,
        }
    }
}

impl TryFrom<NodeResponse> for ConnectPeerResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ConnectPeer {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ListChannelsRequest> for NodeRequest {
    fn from(req: ListChannelsRequest) -> Self {
        NodeRequest::ListChannels {
            pagination: req.pagination.map(|p| p.into()).unwrap_or_default(),
        }
    }
}

impl TryFrom<NodeResponse> for ListChannelsResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ListChannels {
                channels,
                pagination,
            } => {
                let pagination: PaginationResponse = pagination.into();
                Ok(Self {
                    channels: channels
                        .into_iter()
                        .map(|chan| chan.into())
                        .collect::<Vec<ChannelMessage>>(),
                    pagination: Some(pagination),
                })
            }
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ListPaymentsRequest> for NodeRequest {
    fn from(req: ListPaymentsRequest) -> Self {
        NodeRequest::ListPayments {
            pagination: req.pagination.map(|p| p.into()).unwrap_or_default(),
            filter: req.filter.map(|f| f.into()).unwrap_or_default(),
        }
    }
}

impl TryFrom<NodeResponse> for ListPaymentsResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ListPayments {
                payments,
                pagination,
            } => {
                let pagination: PaginationResponse = pagination.into();
                Ok(Self {
                    payments: payments
                        .into_iter()
                        .map(|payment| payment.into())
                        .collect::<Vec<PaymentMessage>>(),
                    pagination: Some(pagination),
                })
            }
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ListPhantomPaymentsRequest> for NodeRequest {
    fn from(req: ListPhantomPaymentsRequest) -> Self {
        NodeRequest::ListPhantomPayments {
            pagination: req.pagination.map(|p| p.into()).unwrap_or_default(),
            filter: req.filter.map(|f| f.into()).unwrap_or_default(),
        }
    }
}

impl TryFrom<NodeResponse> for ListPhantomPaymentsResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ListPhantomPayments {
                payments,
                pagination,
            } => {
                let pagination: PaginationResponse = pagination.into();
                Ok(Self {
                    payments: payments
                        .into_iter()
                        .map(|payment| payment.into())
                        .collect::<Vec<PaymentMessage>>(),
                    pagination: Some(pagination),
                })
            }
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<CloseChannelRequest> for NodeRequest {
    fn from(req: CloseChannelRequest) -> Self {
        NodeRequest::CloseChannel {
            channel_id: req.channel_id,
            force: req.force,
        }
    }
}

impl TryFrom<NodeResponse> for CloseChannelResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::CloseChannel {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<InfoRequest> for NodeRequest {
    fn from(_req: InfoRequest) -> Self {
        NodeRequest::NodeInfo {}
    }
}

impl TryFrom<NodeResponse> for InfoResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::NodeInfo { node_info } => Ok(Self {
                node_info: Some(node_info.into()),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ListPeersRequest> for NodeRequest {
    fn from(_req: ListPeersRequest) -> Self {
        NodeRequest::ListPeers {}
    }
}

impl TryFrom<NodeResponse> for ListPeersResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ListPeers { peers } => Ok(Self {
                peers: peers
                    .into_iter()
                    .map(|peer| peer.into())
                    .collect::<Vec<PeerMessage>>(),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<SignMessageRequest> for NodeRequest {
    fn from(req: SignMessageRequest) -> Self {
        NodeRequest::SignMessage {
            message: req.message,
        }
    }
}

impl TryFrom<NodeResponse> for SignMessageResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::SignMessage { signature } => Ok(Self { signature }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<VerifyMessageRequest> for NodeRequest {
    fn from(req: VerifyMessageRequest) -> Self {
        NodeRequest::VerifyMessage {
            message: req.message,
            signature: req.signature,
        }
    }
}

impl TryFrom<NodeResponse> for VerifyMessageResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::VerifyMessage { valid, pubkey } => Ok(Self { valid, pubkey }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ListUnspentRequest> for NodeRequest {
    fn from(_req: ListUnspentRequest) -> Self {
        NodeRequest::ListUnspent {}
    }
}

impl TryFrom<NodeResponse> for ListUnspentResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ListUnspent { utxos } => Ok(Self {
                utxos: utxos
                    .into_iter()
                    .map(|utxo| utxo.into())
                    .collect::<Vec<UtxoMessage>>(),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<NetworkGraphInfoRequest> for NodeRequest {
    fn from(_req: NetworkGraphInfoRequest) -> Self {
        NodeRequest::NetworkGraphInfo {}
    }
}

impl TryFrom<NodeResponse> for NetworkGraphInfoResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::NetworkGraphInfo {
                num_channels,
                num_nodes,
                num_known_edge_policies,
            } => Ok(Self {
                num_channels,
                num_nodes,
                num_known_edge_policies,
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<entity::peer::Model> for KnownPeer {
    fn from(peer: entity::peer::Model) -> Self {
        Self {
            pubkey: peer.pubkey,
            label: peer.label,
            zero_conf: peer.zero_conf,
        }
    }
}

impl From<ListKnownPeersRequest> for NodeRequest {
    fn from(req: ListKnownPeersRequest) -> Self {
        NodeRequest::ListKnownPeers {
            pagination: req.pagination.map(|p| p.into()).unwrap_or_default(),
        }
    }
}

impl TryFrom<NodeResponse> for ListKnownPeersResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::ListKnownPeers { peers, pagination } => {
                let pagination: PaginationResponse = pagination.into();
                Ok(Self {
                    peers: peers
                        .into_iter()
                        .map(|peer| peer.into())
                        .collect::<Vec<_>>(),
                    pagination: Some(pagination),
                })
            }
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<AddKnownPeerRequest> for NodeRequest {
    fn from(req: AddKnownPeerRequest) -> Self {
        NodeRequest::AddKnownPeer {
            pubkey: req.pubkey,
            label: req.label,
            zero_conf: req.zero_conf,
        }
    }
}

impl TryFrom<NodeResponse> for AddKnownPeerResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::AddKnownPeer {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<RemoveKnownPeerRequest> for NodeRequest {
    fn from(req: RemoveKnownPeerRequest) -> Self {
        NodeRequest::RemoveKnownPeer { pubkey: req.pubkey }
    }
}

impl TryFrom<NodeResponse> for RemoveKnownPeerResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::RemoveKnownPeer {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}
