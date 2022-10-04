// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use tower_cookies::{Cookie, Cookies};

use http::{HeaderValue, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};

use senseicore::{
    services::{
        admin::{AdminRequest, AdminResponse, AdminService, Error, NodeCreateInfo},
        PaginationRequest,
    },
    utils,
};

use super::{auth_header::AuthHeader, utils::get_macaroon_hex_str_from_cookies_or_header};

#[derive(Deserialize)]
pub struct DeleteNodeParams {
    pub pubkey: String,
}

impl From<DeleteNodeParams> for AdminRequest {
    fn from(params: DeleteNodeParams) -> Self {
        Self::DeleteNode {
            pubkey: params.pubkey,
        }
    }
}

#[derive(Deserialize)]
pub struct StartNodeParams {
    pub pubkey: String,
    pub passphrase: String,
}

impl From<StartNodeParams> for AdminRequest {
    fn from(params: StartNodeParams) -> Self {
        Self::StartNode {
            pubkey: params.pubkey,
            passphrase: params.passphrase,
        }
    }
}

#[derive(Deserialize)]
pub struct StopNodeParams {
    pub pubkey: String,
}

impl From<StopNodeParams> for AdminRequest {
    fn from(params: StopNodeParams) -> Self {
        Self::StopNode {
            pubkey: params.pubkey,
        }
    }
}

#[derive(Deserialize)]
pub struct LoginNodeParams {
    pub username: String,
    pub passphrase: String,
}

#[derive(Deserialize)]
pub struct BatchCreateNodeParams {
    nodes: Vec<CreateNodeParams>,
}

impl From<BatchCreateNodeParams> for AdminRequest {
    fn from(params: BatchCreateNodeParams) -> Self {
        Self::BatchCreateNode {
            nodes: params
                .nodes
                .into_iter()
                .map(|node| NodeCreateInfo {
                    username: node.username,
                    alias: node.alias,
                    passphrase: node.passphrase,
                    start: node.start,
                    entropy: node.entropy,
                    cross_node_entropy: node.cross_node_entropy,
                })
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Deserialize)]
pub struct CreateNodeParams {
    pub username: String,
    pub passphrase: String,
    pub alias: String,
    pub start: bool,
    pub entropy: Option<String>,
    pub cross_node_entropy: Option<String>,
}

impl From<CreateNodeParams> for AdminRequest {
    fn from(params: CreateNodeParams) -> Self {
        Self::CreateNode {
            username: params.username,
            passphrase: params.passphrase,
            alias: params.alias,
            start: params.start,
            entropy: params.entropy,
            cross_node_entropy: params.cross_node_entropy,
        }
    }
}

#[derive(Deserialize)]
pub struct ConnectGossipPeerParams {
    node_connection_string: String,
}

impl From<ConnectGossipPeerParams> for AdminRequest {
    fn from(params: ConnectGossipPeerParams) -> Self {
        Self::ConnectGossipPeer {
            node_connection_string: params.node_connection_string,
        }
    }
}

#[derive(Deserialize)]
pub struct FindRouteParams {
    pub payer_public_key_hex: String,
    pub route_params_hex: String,
    pub payment_hash_hex: String,
    pub first_hops: Vec<String>,
}

impl From<FindRouteParams> for AdminRequest {
    fn from(params: FindRouteParams) -> Self {
        Self::FindRoute {
            payer_public_key_hex: params.payer_public_key_hex,
            route_params_hex: params.route_params_hex,
            payment_hash_hex: params.payment_hash_hex,
            first_hops: params.first_hops,
        }
    }
}

#[derive(Deserialize)]
pub struct NodeInfoParams {
    pub node_id_hex: String,
}

impl From<NodeInfoParams> for AdminRequest {
    fn from(params: NodeInfoParams) -> Self {
        Self::NodeInfo {
            node_id_hex: params.node_id_hex,
        }
    }
}

#[derive(Deserialize)]
pub struct PathSuccessfulParams {
    pub path: Vec<String>,
}

impl From<PathSuccessfulParams> for AdminRequest {
    fn from(params: PathSuccessfulParams) -> Self {
        Self::PathSuccessful { path: params.path }
    }
}

#[derive(Deserialize)]
pub struct PathFailedParams {
    pub path: Vec<String>,
    pub short_channel_id: u64,
}

impl From<PathFailedParams> for AdminRequest {
    fn from(params: PathFailedParams) -> Self {
        Self::PathFailed {
            path: params.path,
            short_channel_id: params.short_channel_id,
        }
    }
}

#[derive(Deserialize)]
pub struct GossipNodeAnnouncementParams {
    pub msg_hex: String,
}

impl From<GossipNodeAnnouncementParams> for AdminRequest {
    fn from(params: GossipNodeAnnouncementParams) -> Self {
        Self::GossipNodeAnnouncement {
            msg_hex: params.msg_hex,
        }
    }
}

#[derive(Deserialize)]
pub struct GossipChannelAnnouncementParams {
    pub msg_hex: String,
}

impl From<GossipChannelAnnouncementParams> for AdminRequest {
    fn from(params: GossipChannelAnnouncementParams) -> Self {
        Self::GossipChannelAnnouncement {
            msg_hex: params.msg_hex,
        }
    }
}

#[derive(Deserialize)]
pub struct GossipChannelUpdateParams {
    pub msg_hex: String,
}

impl From<GossipChannelUpdateParams> for AdminRequest {
    fn from(params: GossipChannelUpdateParams) -> Self {
        Self::GossipChannelUpdate {
            msg_hex: params.msg_hex,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateTokenParams {
    pub name: String,
    pub expires_at: u64,
    pub scope: String,
    pub single_use: bool,
}

impl From<CreateTokenParams> for AdminRequest {
    fn from(params: CreateTokenParams) -> Self {
        Self::CreateToken {
            name: params.name,
            expires_at: params.expires_at,
            scope: params.scope,
            single_use: params.single_use,
        }
    }
}

#[derive(Deserialize)]
pub struct DeleteTokenParams {
    pub id: String,
}

impl From<DeleteTokenParams> for AdminRequest {
    fn from(params: DeleteTokenParams) -> Self {
        Self::DeleteToken { id: params.id }
    }
}

#[derive(Deserialize)]
pub struct CreateAdminParams {
    pub username: String,
    pub passphrase: String,
}

impl From<CreateAdminParams> for AdminRequest {
    fn from(params: CreateAdminParams) -> Self {
        Self::CreateAdmin {
            username: params.username,
            passphrase: params.passphrase,
        }
    }
}

pub fn get_token_from_cookies_or_header(
    cookies: &Cookies,
    token: Option<HeaderValue>,
) -> Result<String, StatusCode> {
    match token {
        Some(token) => {
            let res = token
                .to_str()
                .map(|str| str.to_string())
                .map_err(|_| StatusCode::UNAUTHORIZED);
            res
        }
        None => match cookies.get("token") {
            Some(token_cookie) => {
                let token_cookie_str = token_cookie.value().to_string();
                Ok(token_cookie_str)
            }
            None => Err(StatusCode::UNAUTHORIZED),
        },
    }
}

pub async fn authenticate_request(
    admin_service: &AdminService,
    scope: &str,
    cookies: &Cookies,
    token: Option<HeaderValue>,
) -> Result<bool, StatusCode> {
    let token = get_token_from_cookies_or_header(cookies, token)?;

    let access_token = admin_service
        .database
        .get_access_token_by_token(token)
        .await
        .map_err(|_e| StatusCode::UNAUTHORIZED)?;

    match access_token {
        Some(access_token) => {
            if access_token.is_valid(Some(scope)) {
                if access_token.single_use {
                    admin_service
                        .database
                        .delete_access_token(access_token.id)
                        .await
                        .unwrap();
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}

pub fn add_routes(router: Router) -> Router {
    router
        .route("/v1/init", post(init_sensei))
        .route("/v1/nodes", get(list_nodes))
        .route("/v1/nodes", post(create_node))
        .route("/v1/nodes/login", post(login_node))
        .route("/v1/nodes/batch", post(batch_create_nodes))
        .route("/v1/nodes/start", post(start_node))
        .route("/v1/nodes/stop", post(stop_node))
        .route("/v1/nodes/delete", post(delete_node))
        .route("/v1/tokens", get(list_tokens))
        .route("/v1/tokens", post(create_token))
        .route("/v1/tokens", delete(delete_token))
        .route("/v1/status", get(get_status))
        .route("/v1/login", post(login_admin))
        .route("/v1/logout", post(logout))
        .route("/v1/peers/connect", post(connect_gossip_peer))
        .route("/v1/chain/updated", post(chain_updated))
        .route("/v1/ldk/network/route", post(find_route))
        .route("/v1/ldk/network/path/successful", post(path_successful))
        .route("/v1/ldk/network/path/failed", post(path_failed))
        .route("/v1/ldk/network/node_info", post(node_info))
        .route(
            "/v1/ldk/network/gossip/node-announcement",
            post(gossip_node_announcement),
        )
        .route(
            "/v1/ldk/network/gossip/channel-announcement",
            post(gossip_channel_announcement),
        )
        .route(
            "/v1/ldk/network/gossip/channel-update",
            post(gossip_channel_update),
        )
        .route("/v1/ldk/network/graph", get(get_network_graph))
}

pub async fn get_network_graph(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    if authenticated {
        match admin_service.call(AdminRequest::GetNetworkGraph {}).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn connect_gossip_peer(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<ConnectGossipPeerParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn chain_updated(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "chain", &cookies, token).await?;

    if authenticated {
        match admin_service.call(AdminRequest::ChainUpdated {}).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn find_route(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<FindRouteParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn node_info(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<NodeInfoParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn path_successful(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<PathSuccessfulParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn path_failed(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<PathFailedParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn gossip_node_announcement(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<GossipNodeAnnouncementParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn gossip_channel_announcement(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<GossipChannelAnnouncementParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn gossip_channel_update(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "routing", &cookies, token).await?;
    let request = {
        let params: Result<GossipChannelUpdateParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn list_tokens(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Query(pagination): Query<PaginationRequest>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "tokens/list", &cookies, token).await?;
    if authenticated {
        match admin_service
            .call(AdminRequest::ListTokens { pagination })
            .await
        {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn create_token(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "tokens/create", &cookies, token).await?;
    let request = {
        let params: Result<CreateTokenParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn delete_token(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "tokens/delete", &cookies, token).await?;
    let request = {
        let params: Result<DeleteTokenParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn list_nodes(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Query(pagination): Query<PaginationRequest>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "nodes/list", &cookies, token).await?;
    if authenticated {
        match admin_service
            .call(AdminRequest::ListNodes { pagination })
            .await
        {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn login_admin(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let params: LoginNodeParams =
        serde_json::from_value(payload).map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?;

    let admin_user = admin_service
        .database
        .verify_user(params.username, params.passphrase)
        .await
        .map_err(|_e| StatusCode::UNAUTHORIZED)?;
    if admin_user {
        let token = admin_service
            .database
            .get_root_access_token()
            .await
            .map_err(|_e| StatusCode::UNAUTHORIZED)?
            .unwrap();
        let token_cookie = Cookie::build("token", token.token.clone())
            .http_only(true)
            .finish();
        cookies.add(token_cookie);
        Ok(Json(json!({
            "token": token.token
        })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn login_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let params: LoginNodeParams =
        serde_json::from_value(payload).map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?;

    let node = admin_service
        .database
        .get_node_by_username(&params.username)
        .await
        .map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?;

    match node {
        Some(node) => {
            let request = AdminRequest::StartNode {
                pubkey: node.id.clone(),
                passphrase: params.passphrase,
            };
            match admin_service.call(request).await {
                Ok(response) => match response {
                    AdminResponse::StartNode { macaroon } => {
                        let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                            .path("/")
                            .http_only(true)
                            .finish();
                        cookies.add(macaroon_cookie);
                        Ok(Json(json!({
                            "pubkey": node.id,
                            "alias": node.alias,
                            "macaroon": macaroon,
                            "role": node.role as u16
                        })))
                    }
                    _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
                },
                Err(_err) => Err(StatusCode::UNPROCESSABLE_ENTITY),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn logout(cookies: Cookies) -> Result<Json<Value>, StatusCode> {
    cookies.remove(Cookie::new("macaroon", ""));
    cookies.remove(Cookie::new("token", ""));
    Ok(Json::default())
}

pub async fn init_sensei(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let params: Result<CreateAdminParams, _> = serde_json::from_value(payload);

    let request = match params {
        Ok(params) => params.into(),
        Err(_) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "invalid params"})),
            )
        }
    };

    match admin_service.call(request).await {
        Ok(response) => match response {
            AdminResponse::CreateAdmin { token } => {
                let token_cookie = Cookie::build("token", token.clone())
                    .http_only(true)
                    .finish();

                cookies.add(token_cookie);
                (StatusCode::OK, Json(json!({ "token": token })))
            }
            _ => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": "unexpected error"})),
            ),
        },
        Err(err) => match err {
            Error::Generic(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "error": msg })),
            ),
        },
    }
}

// this endpoint is overloaded and serves three purposes
// 1) is the root node created or not
// 2) is the node specified in my macaroon running?
// 3) is my macaroon valid?
pub async fn get_status(
    Extension(admin_service): Extension<Arc<AdminService>>,
    cookies: Cookies,
    AuthHeader { macaroon, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let pubkey = {
        match get_macaroon_hex_str_from_cookies_or_header(&cookies, macaroon) {
            Ok(macaroon_hex) => match utils::macaroon_with_session_from_hex_str(&macaroon_hex) {
                Ok((_macaroon, session)) => Some(session.pubkey),
                Err(_) => None,
            },
            Err(_) => None,
        }
    };

    let authenticated_admin = authenticate_request(&admin_service, "*", &cookies, token)
        .await
        .unwrap_or(false);

    match admin_service
        .call(AdminRequest::GetStatus {
            pubkey,
            authenticated_admin,
        })
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => Ok(Json(AdminResponse::Error(err))),
    }
}

pub async fn create_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "nodes/create", &cookies, token).await?;
    let request = {
        let params: Result<CreateNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn batch_create_nodes(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "nodes/create/batch", &cookies, token).await?;
    let request = {
        let params: Result<BatchCreateNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn start_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "nodes/start", &cookies, token).await?;
    let request = {
        let params: Result<StartNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn stop_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&admin_service, "nodes/stop", &cookies, token).await?;
    let request = {
        let params: Result<StopNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn delete_node(
    Extension(admin_service): Extension<Arc<AdminService>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&admin_service, "nodes/delete", &cookies, token).await?;
    let request = {
        let params: Result<DeleteNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
