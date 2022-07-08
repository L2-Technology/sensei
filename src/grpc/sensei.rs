#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListNode {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub created_at: i64,
    #[prost(int64, tag="3")]
    pub updated_at: i64,
    #[prost(uint32, tag="4")]
    pub role: u32,
    #[prost(string, tag="5")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub alias: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub network: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub listen_addr: ::prost::alloc::string::String,
    #[prost(uint32, tag="9")]
    pub listen_port: u32,
    #[prost(string, tag="10")]
    pub pubkey: ::prost::alloc::string::String,
    #[prost(uint32, tag="11")]
    pub status: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Token {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub created_at: i64,
    #[prost(int64, tag="3")]
    pub updated_at: i64,
    #[prost(int64, tag="4")]
    pub expires_at: i64,
    #[prost(string, tag="5")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub token: ::prost::alloc::string::String,
    #[prost(bool, tag="7")]
    pub single_use: bool,
    #[prost(string, tag="8")]
    pub scope: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaginationRequest {
    #[prost(uint32, tag="1")]
    pub page: u32,
    #[prost(uint32, tag="3")]
    pub take: u32,
    #[prost(string, optional, tag="4")]
    pub query: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaginationResponse {
    #[prost(bool, tag="1")]
    pub has_more: bool,
    #[prost(uint64, tag="2")]
    pub total: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListNodesRequest {
    #[prost(message, optional, tag="1")]
    pub pagination: ::core::option::Option<PaginationRequest>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListNodesResponse {
    #[prost(message, repeated, tag="1")]
    pub nodes: ::prost::alloc::vec::Vec<ListNode>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<PaginationResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTokensRequest {
    #[prost(message, optional, tag="1")]
    pub pagination: ::core::option::Option<PaginationRequest>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTokensResponse {
    #[prost(message, repeated, tag="1")]
    pub tokens: ::prost::alloc::vec::Vec<Token>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<PaginationResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateAdminRequest {
    #[prost(string, tag="1")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub alias: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub passphrase: ::prost::alloc::string::String,
    #[prost(bool, tag="4")]
    pub start: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateAdminResponse {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub macaroon: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub id: ::prost::alloc::string::String,
    #[prost(uint32, tag="4")]
    pub role: u32,
    #[prost(string, tag="5")]
    pub token: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateNodeRequest {
    #[prost(string, tag="1")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub alias: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub passphrase: ::prost::alloc::string::String,
    #[prost(bool, tag="4")]
    pub start: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateNodeResponse {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub macaroon: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub listen_addr: ::prost::alloc::string::String,
    #[prost(int32, tag="4")]
    pub listen_port: i32,
    #[prost(string, tag="5")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteNodeRequest {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteNodeResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTokenRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub scope: ::prost::alloc::string::String,
    #[prost(uint64, tag="3")]
    pub expires_at: u64,
    #[prost(bool, tag="4")]
    pub single_use: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTokenRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTokenResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartAdminRequest {
    #[prost(string, tag="1")]
    pub passphrase: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartAdminResponse {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub macaroon: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub token: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatusRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatusResponse {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub alias: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="3")]
    pub pubkey: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="4")]
    pub username: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag="5")]
    pub role: ::core::option::Option<u32>,
    #[prost(bool, tag="6")]
    pub created: bool,
    #[prost(bool, tag="7")]
    pub running: bool,
    #[prost(bool, tag="8")]
    pub authenticated: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartNodeRequest {
    #[prost(string, tag="1")]
    pub passphrase: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartNodeResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopNodeRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopNodeResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AdminStartNodeRequest {
    #[prost(string, tag="1")]
    pub passphrase: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub pubkey: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AdminStartNodeResponse {
    #[prost(string, tag="1")]
    pub macaroon: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AdminStopNodeRequest {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AdminStopNodeResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUnusedAddressRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUnusedAddressResponse {
    #[prost(string, tag="1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBalanceRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBalanceResponse {
    #[prost(uint64, tag="1")]
    pub onchain_balance_sats: u64,
    #[prost(uint64, tag="2")]
    pub channel_balance_msats: u64,
    #[prost(uint64, tag="3")]
    pub channel_outbound_capacity_msats: u64,
    #[prost(uint64, tag="4")]
    pub channel_inbound_capacity_msats: u64,
    #[prost(uint64, tag="5")]
    pub usable_channel_outbound_capacity_msats: u64,
    #[prost(uint64, tag="6")]
    pub usable_channel_inbound_capacity_msats: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpenChannelRequest {
    #[prost(string, tag="1")]
    pub counterparty_pubkey: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub amount_sats: u64,
    #[prost(bool, tag="3")]
    pub public: bool,
    #[prost(uint64, optional, tag="4")]
    pub push_amount_msats: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag="5")]
    pub custom_id: ::core::option::Option<u64>,
    #[prost(string, optional, tag="6")]
    pub counterparty_host_port: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag="7")]
    pub forwarding_fee_proportional_millionths: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="8")]
    pub forwarding_fee_base_msat: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="9")]
    pub cltv_expiry_delta: ::core::option::Option<u32>,
    #[prost(uint64, optional, tag="10")]
    pub max_dust_htlc_exposure_msat: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag="11")]
    pub force_close_avoidance_max_fee_satoshis: ::core::option::Option<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpenChannelResult {
    #[prost(bool, tag="1")]
    pub error: bool,
    #[prost(string, optional, tag="2")]
    pub error_message: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="3")]
    pub channel_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpenChannelsRequest {
    #[prost(message, repeated, tag="1")]
    pub requests: ::prost::alloc::vec::Vec<OpenChannelRequest>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpenChannelsResponse {
    #[prost(message, repeated, tag="1")]
    pub requests: ::prost::alloc::vec::Vec<OpenChannelRequest>,
    #[prost(message, repeated, tag="2")]
    pub results: ::prost::alloc::vec::Vec<OpenChannelResult>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayInvoiceRequest {
    #[prost(string, tag="1")]
    pub invoice: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayInvoiceResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DecodeInvoiceRequest {
    #[prost(string, tag="1")]
    pub invoice: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DecodeInvoiceResponse {
    #[prost(message, optional, tag="1")]
    pub invoice: ::core::option::Option<Invoice>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Invoice {
    #[prost(string, tag="1")]
    pub payment_hash: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub currency: ::prost::alloc::string::String,
    #[prost(uint64, tag="3")]
    pub amount: u64,
    #[prost(string, tag="4")]
    pub description: ::prost::alloc::string::String,
    #[prost(uint64, tag="5")]
    pub expiry: u64,
    #[prost(uint64, tag="6")]
    pub timestamp: u64,
    #[prost(uint64, tag="7")]
    pub min_final_cltv_expiry: u64,
    #[prost(message, repeated, tag="8")]
    pub route_hints: ::prost::alloc::vec::Vec<RouteHint>,
    #[prost(message, optional, tag="9")]
    pub features: ::core::option::Option<Features>,
    #[prost(string, tag="10")]
    pub payee_pub_key: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RouteHint {
    #[prost(message, repeated, tag="1")]
    pub hops: ::prost::alloc::vec::Vec<RouteHintHop>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RouteHintHop {
    #[prost(string, tag="1")]
    pub src_node_id: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub short_channel_id: u64,
    #[prost(message, optional, tag="3")]
    pub fees: ::core::option::Option<RoutingFees>,
    #[prost(uint32, tag="4")]
    pub cltv_expiry_delta: u32,
    #[prost(uint64, optional, tag="5")]
    pub htlc_minimum_msat: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag="6")]
    pub htlc_maximum_msat: ::core::option::Option<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoutingFees {
    #[prost(uint32, tag="1")]
    pub base_msat: u32,
    #[prost(uint32, tag="2")]
    pub proportional_millionths: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Features {
    #[prost(bool, tag="1")]
    pub variable_length_onion: bool,
    #[prost(bool, tag="2")]
    pub payment_secret: bool,
    #[prost(bool, tag="3")]
    pub basic_mpp: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelPaymentRequest {
    #[prost(string, tag="1")]
    pub label: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub payment_hash: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelPaymentResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeletePaymentRequest {
    #[prost(string, tag="1")]
    pub payment_hash: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeletePaymentResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeysendRequest {
    #[prost(string, tag="1")]
    pub dest_pubkey: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub amt_msat: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeysendResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateInvoiceRequest {
    #[prost(uint64, tag="1")]
    pub amt_msat: u64,
    #[prost(string, tag="2")]
    pub description: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateInvoiceResponse {
    #[prost(string, tag="1")]
    pub invoice: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectPeerRequest {
    #[prost(string, tag="1")]
    pub node_connection_string: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectPeerResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Channel {
    #[prost(string, tag="1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub funding_txid: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag="3")]
    pub funding_tx_index: ::core::option::Option<u32>,
    #[prost(uint64, optional, tag="4")]
    pub short_channel_id: ::core::option::Option<u64>,
    #[prost(uint64, tag="5")]
    pub channel_value_satoshis: u64,
    #[prost(uint64, tag="6")]
    pub balance_msat: u64,
    #[prost(uint64, optional, tag="7")]
    pub unspendable_punishment_reserve: ::core::option::Option<u64>,
    #[prost(uint64, tag="8")]
    pub user_channel_id: u64,
    #[prost(uint64, tag="9")]
    pub outbound_capacity_msat: u64,
    #[prost(uint64, tag="10")]
    pub inbound_capacity_msat: u64,
    #[prost(uint32, optional, tag="11")]
    pub confirmations_required: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag="12")]
    pub force_close_spend_delay: ::core::option::Option<u32>,
    #[prost(bool, tag="13")]
    pub is_outbound: bool,
    #[prost(bool, tag="14")]
    pub is_channel_ready: bool,
    #[prost(bool, tag="15")]
    pub is_usable: bool,
    #[prost(bool, tag="16")]
    pub is_public: bool,
    #[prost(string, tag="17")]
    pub counterparty_pubkey: ::prost::alloc::string::String,
    #[prost(string, optional, tag="18")]
    pub alias: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListChannelsRequest {
    #[prost(message, optional, tag="1")]
    pub pagination: ::core::option::Option<PaginationRequest>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListChannelsResponse {
    #[prost(message, repeated, tag="1")]
    pub channels: ::prost::alloc::vec::Vec<Channel>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<PaginationResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Payment {
    #[prost(string, tag="1")]
    pub hash: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub preimage: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="3")]
    pub secret: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag="4")]
    pub status: ::prost::alloc::string::String,
    #[prost(int64, optional, tag="5")]
    pub amt_msat: ::core::option::Option<i64>,
    #[prost(int64, optional, tag="6")]
    pub fee_paid_msat: ::core::option::Option<i64>,
    #[prost(string, tag="7")]
    pub origin: ::prost::alloc::string::String,
    #[prost(string, optional, tag="8")]
    pub label: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="9")]
    pub invoice: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentsFilter {
    #[prost(string, optional, tag="1")]
    pub origin: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="2")]
    pub status: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPaymentsRequest {
    #[prost(message, optional, tag="1")]
    pub pagination: ::core::option::Option<PaginationRequest>,
    #[prost(message, optional, tag="2")]
    pub filter: ::core::option::Option<PaymentsFilter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPaymentsResponse {
    #[prost(message, repeated, tag="1")]
    pub payments: ::prost::alloc::vec::Vec<Payment>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<PaginationResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloseChannelRequest {
    #[prost(string, tag="1")]
    pub channel_id: ::prost::alloc::string::String,
    #[prost(bool, tag="2")]
    pub force: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloseChannelResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Info {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub node_pubkey: ::prost::alloc::string::String,
    #[prost(uint32, tag="3")]
    pub num_channels: u32,
    #[prost(uint32, tag="4")]
    pub num_usable_channels: u32,
    #[prost(uint32, tag="5")]
    pub num_peers: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InfoRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InfoResponse {
    #[prost(message, optional, tag="1")]
    pub node_info: ::core::option::Option<Info>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Peer {
    #[prost(string, tag="1")]
    pub node_pubkey: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPeersRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPeersResponse {
    #[prost(message, repeated, tag="1")]
    pub peers: ::prost::alloc::vec::Vec<Peer>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignMessageRequest {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignMessageResponse {
    #[prost(string, tag="1")]
    pub signature: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyMessageRequest {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub signature: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyMessageResponse {
    #[prost(bool, tag="1")]
    pub valid: bool,
    #[prost(string, tag="2")]
    pub pubkey: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUnspentRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Utxo {
    #[prost(uint64, tag="1")]
    pub amount_sat: u64,
    #[prost(string, tag="2")]
    pub spk: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub txid: ::prost::alloc::string::String,
    #[prost(uint32, tag="4")]
    pub output_index: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUnspentResponse {
    #[prost(message, repeated, tag="1")]
    pub utxos: ::prost::alloc::vec::Vec<Utxo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkGraphInfoRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkGraphInfoResponse {
    #[prost(uint64, tag="1")]
    pub num_channels: u64,
    #[prost(uint64, tag="2")]
    pub num_nodes: u64,
    #[prost(uint64, tag="3")]
    pub num_known_edge_policies: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddKnownPeerRequest {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub label: ::prost::alloc::string::String,
    #[prost(bool, tag="3")]
    pub zero_conf: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddKnownPeerResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveKnownPeerRequest {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveKnownPeerResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KnownPeer {
    #[prost(string, tag="1")]
    pub pubkey: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub label: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bool, tag="3")]
    pub zero_conf: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListKnownPeersRequest {
    #[prost(message, optional, tag="1")]
    pub pagination: ::core::option::Option<PaginationRequest>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListKnownPeersResponse {
    #[prost(message, repeated, tag="1")]
    pub peers: ::prost::alloc::vec::Vec<KnownPeer>,
    #[prost(message, optional, tag="2")]
    pub pagination: ::core::option::Option<PaginationResponse>,
}
/// Generated client implementations.
pub mod admin_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct AdminClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AdminClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> AdminClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> AdminClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            AdminClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn create_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateAdminRequest>,
        ) -> Result<tonic::Response<super::CreateAdminResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/CreateAdmin");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn start_admin(
            &mut self,
            request: impl tonic::IntoRequest<super::StartAdminRequest>,
        ) -> Result<tonic::Response<super::StartAdminResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/StartAdmin");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_nodes(
            &mut self,
            request: impl tonic::IntoRequest<super::ListNodesRequest>,
        ) -> Result<tonic::Response<super::ListNodesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/ListNodes");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_node(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateNodeRequest>,
        ) -> Result<tonic::Response<super::CreateNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/CreateNode");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_node(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteNodeRequest>,
        ) -> Result<tonic::Response<super::DeleteNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/DeleteNode");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_status(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStatusRequest>,
        ) -> Result<tonic::Response<super::GetStatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/GetStatus");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn start_node(
            &mut self,
            request: impl tonic::IntoRequest<super::AdminStartNodeRequest>,
        ) -> Result<tonic::Response<super::AdminStartNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/StartNode");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn stop_node(
            &mut self,
            request: impl tonic::IntoRequest<super::AdminStopNodeRequest>,
        ) -> Result<tonic::Response<super::AdminStopNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/StopNode");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_tokens(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTokensRequest>,
        ) -> Result<tonic::Response<super::ListTokensResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/ListTokens");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_token(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTokenRequest>,
        ) -> Result<tonic::Response<super::Token>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/CreateToken");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_token(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTokenRequest>,
        ) -> Result<tonic::Response<super::DeleteTokenResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Admin/DeleteToken");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod node_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct NodeClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl NodeClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> NodeClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> NodeClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            NodeClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn start_node(
            &mut self,
            request: impl tonic::IntoRequest<super::StartNodeRequest>,
        ) -> Result<tonic::Response<super::StartNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/StartNode");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn stop_node(
            &mut self,
            request: impl tonic::IntoRequest<super::StopNodeRequest>,
        ) -> Result<tonic::Response<super::StopNodeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/StopNode");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_unused_address(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUnusedAddressRequest>,
        ) -> Result<tonic::Response<super::GetUnusedAddressResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/GetUnusedAddress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_balance(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBalanceRequest>,
        ) -> Result<tonic::Response<super::GetBalanceResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/GetBalance");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn open_channels(
            &mut self,
            request: impl tonic::IntoRequest<super::OpenChannelsRequest>,
        ) -> Result<tonic::Response<super::OpenChannelsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/OpenChannels");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn pay_invoice(
            &mut self,
            request: impl tonic::IntoRequest<super::PayInvoiceRequest>,
        ) -> Result<tonic::Response<super::PayInvoiceResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/PayInvoice");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn decode_invoice(
            &mut self,
            request: impl tonic::IntoRequest<super::DecodeInvoiceRequest>,
        ) -> Result<tonic::Response<super::DecodeInvoiceResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/DecodeInvoice",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn keysend(
            &mut self,
            request: impl tonic::IntoRequest<super::KeysendRequest>,
        ) -> Result<tonic::Response<super::KeysendResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/Keysend");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create_invoice(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateInvoiceRequest>,
        ) -> Result<tonic::Response<super::CreateInvoiceResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/CreateInvoice",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn label_payment(
            &mut self,
            request: impl tonic::IntoRequest<super::LabelPaymentRequest>,
        ) -> Result<tonic::Response<super::LabelPaymentResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/LabelPayment");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_payment(
            &mut self,
            request: impl tonic::IntoRequest<super::DeletePaymentRequest>,
        ) -> Result<tonic::Response<super::DeletePaymentResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/DeletePayment",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn connect_peer(
            &mut self,
            request: impl tonic::IntoRequest<super::ConnectPeerRequest>,
        ) -> Result<tonic::Response<super::ConnectPeerResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/ConnectPeer");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_channels(
            &mut self,
            request: impl tonic::IntoRequest<super::ListChannelsRequest>,
        ) -> Result<tonic::Response<super::ListChannelsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/ListChannels");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_payments(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPaymentsRequest>,
        ) -> Result<tonic::Response<super::ListPaymentsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/ListPayments");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn close_channel(
            &mut self,
            request: impl tonic::IntoRequest<super::CloseChannelRequest>,
        ) -> Result<tonic::Response<super::CloseChannelResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/CloseChannel");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<super::InfoRequest>,
        ) -> Result<tonic::Response<super::InfoResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/Info");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_peers(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPeersRequest>,
        ) -> Result<tonic::Response<super::ListPeersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/ListPeers");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn sign_message(
            &mut self,
            request: impl tonic::IntoRequest<super::SignMessageRequest>,
        ) -> Result<tonic::Response<super::SignMessageResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/SignMessage");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn verify_message(
            &mut self,
            request: impl tonic::IntoRequest<super::VerifyMessageRequest>,
        ) -> Result<tonic::Response<super::VerifyMessageResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/VerifyMessage",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_unspent(
            &mut self,
            request: impl tonic::IntoRequest<super::ListUnspentRequest>,
        ) -> Result<tonic::Response<super::ListUnspentResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/ListUnspent");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn network_graph_info(
            &mut self,
            request: impl tonic::IntoRequest<super::NetworkGraphInfoRequest>,
        ) -> Result<tonic::Response<super::NetworkGraphInfoResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/NetworkGraphInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_known_peers(
            &mut self,
            request: impl tonic::IntoRequest<super::ListKnownPeersRequest>,
        ) -> Result<tonic::Response<super::ListKnownPeersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/ListKnownPeers",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_known_peer(
            &mut self,
            request: impl tonic::IntoRequest<super::AddKnownPeerRequest>,
        ) -> Result<tonic::Response<super::AddKnownPeerResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/sensei.Node/AddKnownPeer");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn remove_known_peer(
            &mut self,
            request: impl tonic::IntoRequest<super::RemoveKnownPeerRequest>,
        ) -> Result<tonic::Response<super::RemoveKnownPeerResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/sensei.Node/RemoveKnownPeer",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod admin_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with AdminServer.
    #[async_trait]
    pub trait Admin: Send + Sync + 'static {
        async fn create_admin(
            &self,
            request: tonic::Request<super::CreateAdminRequest>,
        ) -> Result<tonic::Response<super::CreateAdminResponse>, tonic::Status>;
        async fn start_admin(
            &self,
            request: tonic::Request<super::StartAdminRequest>,
        ) -> Result<tonic::Response<super::StartAdminResponse>, tonic::Status>;
        async fn list_nodes(
            &self,
            request: tonic::Request<super::ListNodesRequest>,
        ) -> Result<tonic::Response<super::ListNodesResponse>, tonic::Status>;
        async fn create_node(
            &self,
            request: tonic::Request<super::CreateNodeRequest>,
        ) -> Result<tonic::Response<super::CreateNodeResponse>, tonic::Status>;
        async fn delete_node(
            &self,
            request: tonic::Request<super::DeleteNodeRequest>,
        ) -> Result<tonic::Response<super::DeleteNodeResponse>, tonic::Status>;
        async fn get_status(
            &self,
            request: tonic::Request<super::GetStatusRequest>,
        ) -> Result<tonic::Response<super::GetStatusResponse>, tonic::Status>;
        async fn start_node(
            &self,
            request: tonic::Request<super::AdminStartNodeRequest>,
        ) -> Result<tonic::Response<super::AdminStartNodeResponse>, tonic::Status>;
        async fn stop_node(
            &self,
            request: tonic::Request<super::AdminStopNodeRequest>,
        ) -> Result<tonic::Response<super::AdminStopNodeResponse>, tonic::Status>;
        async fn list_tokens(
            &self,
            request: tonic::Request<super::ListTokensRequest>,
        ) -> Result<tonic::Response<super::ListTokensResponse>, tonic::Status>;
        async fn create_token(
            &self,
            request: tonic::Request<super::CreateTokenRequest>,
        ) -> Result<tonic::Response<super::Token>, tonic::Status>;
        async fn delete_token(
            &self,
            request: tonic::Request<super::DeleteTokenRequest>,
        ) -> Result<tonic::Response<super::DeleteTokenResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct AdminServer<T: Admin> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Admin> AdminServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AdminServer<T>
    where
        T: Admin,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/sensei.Admin/CreateAdmin" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAdminSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::CreateAdminRequest>
                    for CreateAdminSvc<T> {
                        type Response = super::CreateAdminResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateAdminRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_admin(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateAdminSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/StartAdmin" => {
                    #[allow(non_camel_case_types)]
                    struct StartAdminSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::StartAdminRequest>
                    for StartAdminSvc<T> {
                        type Response = super::StartAdminResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StartAdminRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).start_admin(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StartAdminSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/ListNodes" => {
                    #[allow(non_camel_case_types)]
                    struct ListNodesSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::ListNodesRequest>
                    for ListNodesSvc<T> {
                        type Response = super::ListNodesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListNodesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_nodes(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListNodesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/CreateNode" => {
                    #[allow(non_camel_case_types)]
                    struct CreateNodeSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::CreateNodeRequest>
                    for CreateNodeSvc<T> {
                        type Response = super::CreateNodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/DeleteNode" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteNodeSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::DeleteNodeRequest>
                    for DeleteNodeSvc<T> {
                        type Response = super::DeleteNodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).delete_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/GetStatus" => {
                    #[allow(non_camel_case_types)]
                    struct GetStatusSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::GetStatusRequest>
                    for GetStatusSvc<T> {
                        type Response = super::GetStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_status(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetStatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/StartNode" => {
                    #[allow(non_camel_case_types)]
                    struct StartNodeSvc<T: Admin>(pub Arc<T>);
                    impl<
                        T: Admin,
                    > tonic::server::UnaryService<super::AdminStartNodeRequest>
                    for StartNodeSvc<T> {
                        type Response = super::AdminStartNodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AdminStartNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).start_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StartNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/StopNode" => {
                    #[allow(non_camel_case_types)]
                    struct StopNodeSvc<T: Admin>(pub Arc<T>);
                    impl<
                        T: Admin,
                    > tonic::server::UnaryService<super::AdminStopNodeRequest>
                    for StopNodeSvc<T> {
                        type Response = super::AdminStopNodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AdminStopNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).stop_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StopNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/ListTokens" => {
                    #[allow(non_camel_case_types)]
                    struct ListTokensSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::ListTokensRequest>
                    for ListTokensSvc<T> {
                        type Response = super::ListTokensResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTokensRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_tokens(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListTokensSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/CreateToken" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTokenSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::CreateTokenRequest>
                    for CreateTokenSvc<T> {
                        type Response = super::Token;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateTokenRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_token(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateTokenSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Admin/DeleteToken" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTokenSvc<T: Admin>(pub Arc<T>);
                    impl<T: Admin> tonic::server::UnaryService<super::DeleteTokenRequest>
                    for DeleteTokenSvc<T> {
                        type Response = super::DeleteTokenResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTokenRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_token(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteTokenSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Admin> Clone for AdminServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Admin> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Admin> tonic::transport::NamedService for AdminServer<T> {
        const NAME: &'static str = "sensei.Admin";
    }
}
/// Generated server implementations.
pub mod node_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with NodeServer.
    #[async_trait]
    pub trait Node: Send + Sync + 'static {
        async fn start_node(
            &self,
            request: tonic::Request<super::StartNodeRequest>,
        ) -> Result<tonic::Response<super::StartNodeResponse>, tonic::Status>;
        async fn stop_node(
            &self,
            request: tonic::Request<super::StopNodeRequest>,
        ) -> Result<tonic::Response<super::StopNodeResponse>, tonic::Status>;
        async fn get_unused_address(
            &self,
            request: tonic::Request<super::GetUnusedAddressRequest>,
        ) -> Result<tonic::Response<super::GetUnusedAddressResponse>, tonic::Status>;
        async fn get_balance(
            &self,
            request: tonic::Request<super::GetBalanceRequest>,
        ) -> Result<tonic::Response<super::GetBalanceResponse>, tonic::Status>;
        async fn open_channels(
            &self,
            request: tonic::Request<super::OpenChannelsRequest>,
        ) -> Result<tonic::Response<super::OpenChannelsResponse>, tonic::Status>;
        async fn pay_invoice(
            &self,
            request: tonic::Request<super::PayInvoiceRequest>,
        ) -> Result<tonic::Response<super::PayInvoiceResponse>, tonic::Status>;
        async fn decode_invoice(
            &self,
            request: tonic::Request<super::DecodeInvoiceRequest>,
        ) -> Result<tonic::Response<super::DecodeInvoiceResponse>, tonic::Status>;
        async fn keysend(
            &self,
            request: tonic::Request<super::KeysendRequest>,
        ) -> Result<tonic::Response<super::KeysendResponse>, tonic::Status>;
        async fn create_invoice(
            &self,
            request: tonic::Request<super::CreateInvoiceRequest>,
        ) -> Result<tonic::Response<super::CreateInvoiceResponse>, tonic::Status>;
        async fn label_payment(
            &self,
            request: tonic::Request<super::LabelPaymentRequest>,
        ) -> Result<tonic::Response<super::LabelPaymentResponse>, tonic::Status>;
        async fn delete_payment(
            &self,
            request: tonic::Request<super::DeletePaymentRequest>,
        ) -> Result<tonic::Response<super::DeletePaymentResponse>, tonic::Status>;
        async fn connect_peer(
            &self,
            request: tonic::Request<super::ConnectPeerRequest>,
        ) -> Result<tonic::Response<super::ConnectPeerResponse>, tonic::Status>;
        async fn list_channels(
            &self,
            request: tonic::Request<super::ListChannelsRequest>,
        ) -> Result<tonic::Response<super::ListChannelsResponse>, tonic::Status>;
        async fn list_payments(
            &self,
            request: tonic::Request<super::ListPaymentsRequest>,
        ) -> Result<tonic::Response<super::ListPaymentsResponse>, tonic::Status>;
        async fn close_channel(
            &self,
            request: tonic::Request<super::CloseChannelRequest>,
        ) -> Result<tonic::Response<super::CloseChannelResponse>, tonic::Status>;
        async fn info(
            &self,
            request: tonic::Request<super::InfoRequest>,
        ) -> Result<tonic::Response<super::InfoResponse>, tonic::Status>;
        async fn list_peers(
            &self,
            request: tonic::Request<super::ListPeersRequest>,
        ) -> Result<tonic::Response<super::ListPeersResponse>, tonic::Status>;
        async fn sign_message(
            &self,
            request: tonic::Request<super::SignMessageRequest>,
        ) -> Result<tonic::Response<super::SignMessageResponse>, tonic::Status>;
        async fn verify_message(
            &self,
            request: tonic::Request<super::VerifyMessageRequest>,
        ) -> Result<tonic::Response<super::VerifyMessageResponse>, tonic::Status>;
        async fn list_unspent(
            &self,
            request: tonic::Request<super::ListUnspentRequest>,
        ) -> Result<tonic::Response<super::ListUnspentResponse>, tonic::Status>;
        async fn network_graph_info(
            &self,
            request: tonic::Request<super::NetworkGraphInfoRequest>,
        ) -> Result<tonic::Response<super::NetworkGraphInfoResponse>, tonic::Status>;
        async fn list_known_peers(
            &self,
            request: tonic::Request<super::ListKnownPeersRequest>,
        ) -> Result<tonic::Response<super::ListKnownPeersResponse>, tonic::Status>;
        async fn add_known_peer(
            &self,
            request: tonic::Request<super::AddKnownPeerRequest>,
        ) -> Result<tonic::Response<super::AddKnownPeerResponse>, tonic::Status>;
        async fn remove_known_peer(
            &self,
            request: tonic::Request<super::RemoveKnownPeerRequest>,
        ) -> Result<tonic::Response<super::RemoveKnownPeerResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct NodeServer<T: Node> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Node> NodeServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for NodeServer<T>
    where
        T: Node,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/sensei.Node/StartNode" => {
                    #[allow(non_camel_case_types)]
                    struct StartNodeSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::StartNodeRequest>
                    for StartNodeSvc<T> {
                        type Response = super::StartNodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StartNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).start_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StartNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/StopNode" => {
                    #[allow(non_camel_case_types)]
                    struct StopNodeSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::StopNodeRequest>
                    for StopNodeSvc<T> {
                        type Response = super::StopNodeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StopNodeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).stop_node(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StopNodeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/GetUnusedAddress" => {
                    #[allow(non_camel_case_types)]
                    struct GetUnusedAddressSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::GetUnusedAddressRequest>
                    for GetUnusedAddressSvc<T> {
                        type Response = super::GetUnusedAddressResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUnusedAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_unused_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUnusedAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/GetBalance" => {
                    #[allow(non_camel_case_types)]
                    struct GetBalanceSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::GetBalanceRequest>
                    for GetBalanceSvc<T> {
                        type Response = super::GetBalanceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBalanceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_balance(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBalanceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/OpenChannels" => {
                    #[allow(non_camel_case_types)]
                    struct OpenChannelsSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::OpenChannelsRequest>
                    for OpenChannelsSvc<T> {
                        type Response = super::OpenChannelsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OpenChannelsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).open_channels(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OpenChannelsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/PayInvoice" => {
                    #[allow(non_camel_case_types)]
                    struct PayInvoiceSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::PayInvoiceRequest>
                    for PayInvoiceSvc<T> {
                        type Response = super::PayInvoiceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PayInvoiceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).pay_invoice(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PayInvoiceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/DecodeInvoice" => {
                    #[allow(non_camel_case_types)]
                    struct DecodeInvoiceSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::DecodeInvoiceRequest>
                    for DecodeInvoiceSvc<T> {
                        type Response = super::DecodeInvoiceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DecodeInvoiceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).decode_invoice(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DecodeInvoiceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/Keysend" => {
                    #[allow(non_camel_case_types)]
                    struct KeysendSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::KeysendRequest>
                    for KeysendSvc<T> {
                        type Response = super::KeysendResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KeysendRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).keysend(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = KeysendSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/CreateInvoice" => {
                    #[allow(non_camel_case_types)]
                    struct CreateInvoiceSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::CreateInvoiceRequest>
                    for CreateInvoiceSvc<T> {
                        type Response = super::CreateInvoiceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateInvoiceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_invoice(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateInvoiceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/LabelPayment" => {
                    #[allow(non_camel_case_types)]
                    struct LabelPaymentSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::LabelPaymentRequest>
                    for LabelPaymentSvc<T> {
                        type Response = super::LabelPaymentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LabelPaymentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).label_payment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LabelPaymentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/DeletePayment" => {
                    #[allow(non_camel_case_types)]
                    struct DeletePaymentSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::DeletePaymentRequest>
                    for DeletePaymentSvc<T> {
                        type Response = super::DeletePaymentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeletePaymentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_payment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeletePaymentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/ConnectPeer" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectPeerSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::ConnectPeerRequest>
                    for ConnectPeerSvc<T> {
                        type Response = super::ConnectPeerResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ConnectPeerRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).connect_peer(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectPeerSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/ListChannels" => {
                    #[allow(non_camel_case_types)]
                    struct ListChannelsSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::ListChannelsRequest>
                    for ListChannelsSvc<T> {
                        type Response = super::ListChannelsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListChannelsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_channels(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListChannelsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/ListPayments" => {
                    #[allow(non_camel_case_types)]
                    struct ListPaymentsSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::ListPaymentsRequest>
                    for ListPaymentsSvc<T> {
                        type Response = super::ListPaymentsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListPaymentsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_payments(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListPaymentsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/CloseChannel" => {
                    #[allow(non_camel_case_types)]
                    struct CloseChannelSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::CloseChannelRequest>
                    for CloseChannelSvc<T> {
                        type Response = super::CloseChannelResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CloseChannelRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).close_channel(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CloseChannelSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/Info" => {
                    #[allow(non_camel_case_types)]
                    struct InfoSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::InfoRequest>
                    for InfoSvc<T> {
                        type Response = super::InfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/ListPeers" => {
                    #[allow(non_camel_case_types)]
                    struct ListPeersSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::ListPeersRequest>
                    for ListPeersSvc<T> {
                        type Response = super::ListPeersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListPeersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_peers(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListPeersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/SignMessage" => {
                    #[allow(non_camel_case_types)]
                    struct SignMessageSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::SignMessageRequest>
                    for SignMessageSvc<T> {
                        type Response = super::SignMessageResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SignMessageRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).sign_message(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SignMessageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/VerifyMessage" => {
                    #[allow(non_camel_case_types)]
                    struct VerifyMessageSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::VerifyMessageRequest>
                    for VerifyMessageSvc<T> {
                        type Response = super::VerifyMessageResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::VerifyMessageRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).verify_message(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VerifyMessageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/ListUnspent" => {
                    #[allow(non_camel_case_types)]
                    struct ListUnspentSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::ListUnspentRequest>
                    for ListUnspentSvc<T> {
                        type Response = super::ListUnspentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListUnspentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_unspent(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListUnspentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/NetworkGraphInfo" => {
                    #[allow(non_camel_case_types)]
                    struct NetworkGraphInfoSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::NetworkGraphInfoRequest>
                    for NetworkGraphInfoSvc<T> {
                        type Response = super::NetworkGraphInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NetworkGraphInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).network_graph_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NetworkGraphInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/ListKnownPeers" => {
                    #[allow(non_camel_case_types)]
                    struct ListKnownPeersSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::ListKnownPeersRequest>
                    for ListKnownPeersSvc<T> {
                        type Response = super::ListKnownPeersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListKnownPeersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_known_peers(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListKnownPeersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/AddKnownPeer" => {
                    #[allow(non_camel_case_types)]
                    struct AddKnownPeerSvc<T: Node>(pub Arc<T>);
                    impl<T: Node> tonic::server::UnaryService<super::AddKnownPeerRequest>
                    for AddKnownPeerSvc<T> {
                        type Response = super::AddKnownPeerResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddKnownPeerRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).add_known_peer(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddKnownPeerSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/sensei.Node/RemoveKnownPeer" => {
                    #[allow(non_camel_case_types)]
                    struct RemoveKnownPeerSvc<T: Node>(pub Arc<T>);
                    impl<
                        T: Node,
                    > tonic::server::UnaryService<super::RemoveKnownPeerRequest>
                    for RemoveKnownPeerSvc<T> {
                        type Response = super::RemoveKnownPeerResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RemoveKnownPeerRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).remove_known_peer(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RemoveKnownPeerSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Node> Clone for NodeServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Node> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Node> tonic::transport::NamedService for NodeServer<T> {
        const NAME: &'static str = "sensei.Node";
    }
}
