// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::sync::Arc;

use crate::http::auth_header::AuthHeader;
use crate::AdminService;
use axum::extract::{Extension, Json, Query};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use http::{HeaderValue, StatusCode};
use senseicore::services::admin::AdminRequest;
use senseicore::services::node::{NodeRequest, NodeRequestError, NodeResponse, OpenChannelRequest};
use senseicore::services::{
    ListChannelsParams, ListKnownPeersParams, ListPaymentsParams, ListTransactionsParams,
};
use senseicore::utils;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;

use super::utils::get_macaroon_hex_str_from_cookies_or_header;

#[derive(Deserialize)]
pub struct GetInvoiceParams {
    pub amt_msat: u64,
    pub description: String,
}

impl From<GetInvoiceParams> for NodeRequest {
    fn from(params: GetInvoiceParams) -> Self {
        Self::GetInvoice {
            amt_msat: params.amt_msat,
            description: params.description,
        }
    }
}

#[derive(Deserialize)]
pub struct GetPhantomInvoiceParams {
    pub amt_msat: u64,
    pub description: String,
    pub phantom_route_hints_hex: Vec<String>,
}

impl From<GetPhantomInvoiceParams> for NodeRequest {
    fn from(params: GetPhantomInvoiceParams) -> Self {
        Self::GetPhantomInvoice {
            amt_msat: params.amt_msat,
            description: params.description,
            phantom_route_hints_hex: params.phantom_route_hints_hex,
        }
    }
}

#[derive(Deserialize)]
pub struct LabelPaymentParams {
    pub label: String,
    pub payment_hash: String,
}

impl From<LabelPaymentParams> for NodeRequest {
    fn from(params: LabelPaymentParams) -> Self {
        Self::LabelPayment {
            label: params.label,
            payment_hash: params.payment_hash,
        }
    }
}

#[derive(Deserialize)]
pub struct DeletePaymentParams {
    pub payment_hash: String,
}

impl From<DeletePaymentParams> for NodeRequest {
    fn from(params: DeletePaymentParams) -> Self {
        Self::DeletePayment {
            payment_hash: params.payment_hash,
        }
    }
}

#[derive(Deserialize)]
pub struct BatchOpenChannelParams {
    requests: Vec<OpenChannelRequest>,
}

impl From<BatchOpenChannelParams> for NodeRequest {
    fn from(params: BatchOpenChannelParams) -> Self {
        Self::OpenChannels {
            requests: params.requests,
        }
    }
}

#[derive(Deserialize)]
pub struct SendPaymentParams {
    pub invoice: String,
}

impl From<SendPaymentParams> for NodeRequest {
    fn from(params: SendPaymentParams) -> Self {
        Self::SendPayment {
            invoice: params.invoice,
        }
    }
}

#[derive(Deserialize)]
pub struct DecodeInvoiceParams {
    pub invoice: String,
}

impl From<DecodeInvoiceParams> for NodeRequest {
    fn from(params: DecodeInvoiceParams) -> Self {
        Self::DecodeInvoice {
            invoice: params.invoice,
        }
    }
}

#[derive(Deserialize)]
pub struct KeysendParams {
    pub dest_pubkey: String,
    pub amt_msat: u64,
}

impl From<KeysendParams> for NodeRequest {
    fn from(params: KeysendParams) -> Self {
        Self::Keysend {
            dest_pubkey: params.dest_pubkey,
            amt_msat: params.amt_msat,
        }
    }
}

#[derive(Deserialize)]
pub struct ConnectPeerParams {
    pub node_connection_string: String,
}

impl From<ConnectPeerParams> for NodeRequest {
    fn from(params: ConnectPeerParams) -> Self {
        Self::ConnectPeer {
            node_connection_string: params.node_connection_string,
        }
    }
}

#[derive(Deserialize)]
pub struct CloseChannelParams {
    pub channel_id: String,
    pub force: bool,
}

impl From<CloseChannelParams> for NodeRequest {
    fn from(params: CloseChannelParams) -> Self {
        Self::CloseChannel {
            channel_id: params.channel_id,
            force: params.force,
        }
    }
}

#[derive(Deserialize)]
pub struct StartNodeParams {
    pub passphrase: String,
}

impl From<StartNodeParams> for NodeRequest {
    fn from(params: StartNodeParams) -> Self {
        Self::StartNode {
            passphrase: params.passphrase,
        }
    }
}

#[derive(Deserialize)]
pub struct SignMessageParams {
    pub message: String,
}

impl From<SignMessageParams> for NodeRequest {
    fn from(params: SignMessageParams) -> Self {
        Self::SignMessage {
            message: params.message,
        }
    }
}

#[derive(Deserialize)]
pub struct VerifyMessageParams {
    pub message: String,
    pub signature: String,
}

impl From<VerifyMessageParams> for NodeRequest {
    fn from(params: VerifyMessageParams) -> Self {
        Self::VerifyMessage {
            message: params.message,
            signature: params.signature,
        }
    }
}

#[derive(Deserialize)]
pub struct AddKnownPeerParams {
    pub pubkey: String,
    pub label: String,
    pub zero_conf: bool,
}

impl From<AddKnownPeerParams> for NodeRequest {
    fn from(params: AddKnownPeerParams) -> Self {
        Self::AddKnownPeer {
            pubkey: params.pubkey,
            label: params.label,
            zero_conf: params.zero_conf,
        }
    }
}

#[derive(Deserialize)]
pub struct RemoveKnownPeerParams {
    pub pubkey: String,
}

impl From<RemoveKnownPeerParams> for NodeRequest {
    fn from(params: RemoveKnownPeerParams) -> Self {
        Self::RemoveKnownPeer {
            pubkey: params.pubkey,
        }
    }
}

pub fn add_routes(router: Router) -> Router {
    router
        .route("/v1/node/payments", get(handle_get_payments))
        .route(
            "/v1/node/phantom-payments",
            get(handle_get_phantom_payments),
        )
        .route("/v1/node/wallet/address", get(get_unused_address))
        .route("/v1/node/wallet/balance", get(get_wallet_balance))
        .route("/v1/node/wallet/utxos", get(list_unspent))
        .route("/v1/node/channels", get(get_channels))
        .route("/v1/node/transactions", get(get_transactions))
        .route("/v1/node/info", get(get_info))
        .route("/v1/node/peers", get(get_peers))
        .route("/v1/node/stop", get(stop_node))
        .route("/v1/node/start", post(start_node))
        .route("/v1/node/invoices", post(create_invoice))
        .route("/v1/node/invoices/phantom", post(create_phantom_invoice))
        .route("/v1/node/invoices/pay", post(pay_invoice))
        .route("/v1/node/invoices/decode", post(decode_invoice))
        .route("/v1/node/payments/label", post(label_payment))
        .route("/v1/node/payments/delete", post(delete_payment))
        .route("/v1/node/channels/open", post(open_channels))
        .route("/v1/node/channels/close", post(close_channel))
        .route("/v1/node/keysend", post(keysend))
        .route("/v1/node/peers/connect", post(connect_peer))
        .route("/v1/node/sign/message", post(sign_message))
        .route("/v1/node/verify/message", post(verify_message))
        .route("/v1/node/network-graph/info", get(network_graph_info))
        .route("/v1/node/known-peers", get(list_known_peers))
        .route("/v1/node/known-peers", post(add_known_peer))
        .route("/v1/node/known-peers", delete(remove_known_peer))
        .route("/v1/node/ldk/phantom-route-hints", get(phantom_route_hints))
}

pub async fn phantom_route_hints(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(
        admin_service,
        NodeRequest::GetPhantomRouteHints {},
        macaroon,
        cookies,
    )
    .await
}

pub async fn get_unused_address(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(
        admin_service,
        NodeRequest::GetUnusedAddress {},
        macaroon,
        cookies,
    )
    .await
}

pub async fn get_wallet_balance(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(admin_service, NodeRequest::GetBalance {}, macaroon, cookies).await
}

pub async fn handle_get_payments(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Query(params): Query<ListPaymentsParams>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = NodeRequest::ListPayments {
        pagination: params.clone().into(),
        filter: params.into(),
    };

    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn handle_get_phantom_payments(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Query(params): Query<ListPaymentsParams>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = NodeRequest::ListPhantomPayments {
        pagination: params.clone().into(),
        filter: params.into(),
    };

    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn get_channels(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Query(params): Query<ListChannelsParams>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = NodeRequest::ListChannels {
        pagination: params.clone().into(),
    };

    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn get_transactions(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Query(params): Query<ListTransactionsParams>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = NodeRequest::ListTransactions {
        pagination: params.clone().into(),
    };

    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn get_info(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(admin_service, NodeRequest::NodeInfo {}, macaroon, cookies).await
}

pub async fn get_peers(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(admin_service, NodeRequest::ListPeers {}, macaroon, cookies).await
}

pub async fn stop_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(admin_service, NodeRequest::StopNode {}, macaroon, cookies).await
}

pub async fn handle_authenticated_request(
    admin_service: Arc<AdminService>,
    request: NodeRequest,
    macaroon: Option<HeaderValue>,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let macaroon_hex_string = get_macaroon_hex_str_from_cookies_or_header(&cookies, macaroon)?;

    let (macaroon, session) = utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
        .map_err(|_e| (StatusCode::UNAUTHORIZED, "unauthorized").into_response())?;

    let pubkey = session.pubkey.clone();
    let node_directory = admin_service.node_directory.lock().await;
    let node = node_directory.get(&session.pubkey);

    match node {
        Some(Some(handle)) => {
            handle
                .node
                .verify_macaroon(macaroon, session)
                .await
                .map_err(|_e| (StatusCode::UNAUTHORIZED, "unauthorized").into_response())?;

            match request {
                NodeRequest::StopNode {} => {
                    let admin_request = AdminRequest::StopNode { pubkey };
                    let _ = admin_service.call(admin_request).await.map_err(|_e| {
                        (StatusCode::UNPROCESSABLE_ENTITY, "error").into_response()
                    })?;
                    Ok(Json(NodeResponse::StopNode {}))
                }
                _ => match handle.node.call(request).await {
                    Ok(response) => Ok(Json(response)),
                    Err(err) => {
                        let error_message = match err {
                            NodeRequestError::Sensei(e) => e,
                            NodeRequestError::Bdk(e) => e,
                            NodeRequestError::Io(e) => e,
                        };

                        let body = Json(json!({
                            "error": error_message,
                        }));

                        Err((StatusCode::UNPROCESSABLE_ENTITY, body).into_response())
                    }
                },
            }
        }
        Some(None) => {
            // TODO: rethink this Some(None) business
            let err = senseicore::error::Error::Unauthenticated;
            let node_request_error: NodeRequestError = err.into();
            Ok(Json(NodeResponse::Error(node_request_error)))
        }
        None => match request {
            NodeRequest::StartNode { passphrase } => {
                drop(node_directory);
                let req = AdminRequest::StartNode {
                    passphrase,
                    pubkey: session.pubkey,
                };
                let _ = admin_service
                    .call(req)
                    .await
                    .map_err(|_e| (StatusCode::UNAUTHORIZED, "unauthorized").into_response())?;
                Ok(Json(NodeResponse::StartNode {}))
            }
            _ => {
                let err = senseicore::error::Error::Unauthenticated;
                let node_request_error: NodeRequestError = err.into();
                Ok(Json(NodeResponse::Error(node_request_error)))
            }
        },
    }
}

pub async fn start_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<StartNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn create_invoice(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<GetInvoiceParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn create_phantom_invoice(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<GetPhantomInvoiceParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn label_payment(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<LabelPaymentParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn delete_payment(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<DeletePaymentParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn pay_invoice(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<SendPaymentParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn decode_invoice(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<DecodeInvoiceParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn open_channels(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<BatchOpenChannelParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn close_channel(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<CloseChannelParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn keysend(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<KeysendParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn connect_peer(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<ConnectPeerParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn sign_message(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<SignMessageParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn verify_message(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<VerifyMessageParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn list_unspent(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(
        admin_service,
        NodeRequest::ListUnspent {},
        macaroon,
        cookies,
    )
    .await
}

pub async fn network_graph_info(
    Extension(admin_service): Extension<Arc<AdminService>>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    handle_authenticated_request(
        admin_service,
        NodeRequest::NetworkGraphInfo {},
        macaroon,
        cookies,
    )
    .await
}

pub async fn list_known_peers(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Query(params): Query<ListKnownPeersParams>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = NodeRequest::ListKnownPeers {
        pagination: params.clone().into(),
    };

    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn add_known_peer(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<AddKnownPeerParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}

pub async fn remove_known_peer(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    AuthHeader { macaroon, token: _ }: AuthHeader,
    cookies: Cookies,
) -> Result<Json<NodeResponse>, Response> {
    let request = {
        let params: Result<RemoveKnownPeerParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid params").into_response()),
        }
    }?;
    handle_authenticated_request(admin_service, request, macaroon, cookies).await
}
