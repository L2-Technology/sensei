use super::sensei::{
    Channel as ChannelMessage, DeletePaymentRequest, DeletePaymentResponse, Info as InfoMessage,
    LabelPaymentRequest, LabelPaymentResponse, PaginationRequest, PaginationResponse,
    Payment as PaymentMessage, PaymentsFilter, Peer as PeerMessage, StartNodeRequest,
    StartNodeResponse, StopNodeRequest, StopNodeResponse,
};

use super::sensei::{
    CloseChannelRequest, CloseChannelResponse, ConnectPeerRequest, ConnectPeerResponse,
    CreateInvoiceRequest, CreateInvoiceResponse, GetBalanceRequest, GetBalanceResponse,
    GetUnusedAddressRequest, GetUnusedAddressResponse, InfoRequest, InfoResponse, KeysendRequest,
    KeysendResponse, ListChannelsRequest, ListChannelsResponse, ListPaymentsRequest,
    ListPaymentsResponse, ListPeersRequest, ListPeersResponse, OpenChannelRequest,
    OpenChannelResponse, PayInvoiceRequest, PayInvoiceResponse, SignMessageRequest,
    SignMessageResponse,
};

use crate::database::node::Payment;
use crate::services::{
    self,
    node::{Channel, NodeInfo, NodeRequest, NodeResponse, Peer},
};

impl From<Option<PaymentsFilter>> for services::PaymentsFilter {
    fn from(filter: Option<PaymentsFilter>) -> Self {
        match filter {
            Some(filter) => Self {
                origin: filter.origin,
                status: filter.status,
            },
            None => Self::default(),
        }
    }
}

impl From<Option<PaginationRequest>> for services::PaginationRequest {
    fn from(pagination: Option<PaginationRequest>) -> Self {
        match pagination {
            Some(pagination) => Self {
                take: pagination.take,
                page: pagination.page,
                query: pagination.query,
            },
            None => Self::default(),
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
            is_funding_locked: channel.is_funding_locked,
            is_usable: channel.is_usable,
            is_public: channel.is_public,
            counterparty_pubkey: channel.counterparty_pubkey,
            alias: channel.alias,
        }
    }
}

impl From<Payment> for PaymentMessage {
    fn from(payment: Payment) -> Self {
        Self {
            hash: payment.payment_hash,
            preimage: payment.preimage,
            secret: payment.secret,
            status: payment.status,
            amt_msat: payment.amt_msat,
            origin: payment.origin,
            label: payment.label,
            invoice: payment.invoice,
        }
    }
}

impl From<NodeInfo> for InfoMessage {
    fn from(info: NodeInfo) -> Self {
        Self {
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
            NodeResponse::GetBalance { balance_satoshis } => Ok(Self { balance_satoshis }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<OpenChannelRequest> for NodeRequest {
    fn from(req: OpenChannelRequest) -> Self {
        NodeRequest::OpenChannel {
            node_connection_string: req.node_connection_string,
            amt_satoshis: req.amt_satoshis,
            public: req.public,
        }
    }
}

impl TryFrom<NodeResponse> for OpenChannelResponse {
    type Error = String;

    fn try_from(res: NodeResponse) -> Result<Self, Self::Error> {
        match res {
            NodeResponse::OpenChannel {} => Ok(Self {}),
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
            pagination: req.pagination.into(),
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
            pagination: req.pagination.into(),
            filter: req.filter.into(),
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
