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
    routing::{delete, get, post},
    Json, Router,
};
use tower_cookies::{Cookie, Cookies};

use http::{HeaderValue, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    services::{
        admin::{AdminRequest, AdminResponse},
        PaginationRequest,
    },
    utils, RequestContext,
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
pub struct CreateNodeParams {
    pub username: String,
    pub passphrase: String,
    pub alias: String,
    pub start: bool,
}

impl From<CreateNodeParams> for AdminRequest {
    fn from(params: CreateNodeParams) -> Self {
        Self::CreateNode {
            username: params.username,
            passphrase: params.passphrase,
            alias: params.alias,
            start: params.start,
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
    pub id: u64,
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
    pub alias: String,
    pub start: bool,
}

impl From<CreateAdminParams> for AdminRequest {
    fn from(params: CreateAdminParams) -> Self {
        Self::CreateAdmin {
            username: params.username,
            passphrase: params.passphrase,
            alias: params.alias,
            start: params.start,
        }
    }
}

#[derive(Deserialize)]
pub struct StartAdminParams {
    pub passphrase: String,
}

impl From<StartAdminParams> for AdminRequest {
    fn from(params: StartAdminParams) -> Self {
        Self::StartAdmin {
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
    request_context: &RequestContext,
    scope: &str,
    cookies: &Cookies,
    token: Option<HeaderValue>,
) -> Result<bool, StatusCode> {
    let token = get_token_from_cookies_or_header(cookies, token)?;

    let access_token = {
        let mut database = request_context.admin_service.database.lock().await;
        database.get_access_token(token)
    }
    .map_err(|_e| StatusCode::UNAUTHORIZED)?;

    match access_token {
        Some(access_token) => {
            if access_token.is_valid(Some(scope)) {
                if access_token.single_use {
                    let mut database = request_context.admin_service.database.lock().await;
                    database.delete_access_token(access_token.id).unwrap();
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
        .route("/v1/nodes/start", post(start_node))
        .route("/v1/nodes/stop", post(stop_node))
        .route("/v1/nodes/delete", post(delete_node))
        .route("/v1/tokens", get(list_tokens))
        .route("/v1/tokens", post(create_token))
        .route("/v1/tokens", delete(delete_token))
        .route("/v1/status", get(get_status))
        .route("/v1/start", post(start_sensei))
        .route("/v1/login", post(login))
        .route("/v1/logout", post(logout))
}

pub async fn list_tokens(
    Extension(request_context): Extension<Arc<RequestContext>>,
    cookies: Cookies,
    Query(pagination): Query<PaginationRequest>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "tokens/list", &cookies, token).await?;
    if authenticated {
        match request_context
            .admin_service
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
    Extension(request_context): Extension<Arc<RequestContext>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "tokens/create", &cookies, token).await?;
    let request = {
        let params: Result<CreateTokenParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match request_context.admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn delete_token(
    Extension(request_context): Extension<Arc<RequestContext>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "tokens/delete", &cookies, token).await?;
    let request = {
        let params: Result<DeleteTokenParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match request_context.admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn list_nodes(
    Extension(request_context): Extension<Arc<RequestContext>>,
    cookies: Cookies,
    Query(pagination): Query<PaginationRequest>,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "nodes/list", &cookies, token).await?;
    if authenticated {
        match request_context
            .admin_service
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

pub async fn login(
    Extension(request_context): Extension<Arc<RequestContext>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let params: LoginNodeParams =
        serde_json::from_value(payload).map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?;

    let node = {
        let mut database = request_context.admin_service.database.lock().await;
        database
            .get_node_by_username(params.username)
            .map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?
    };

    match node {
        Some(node) => {
            let request = match node.is_admin() {
                true => AdminRequest::StartAdmin {
                    passphrase: params.passphrase,
                },
                false => AdminRequest::StartNode {
                    pubkey: node.pubkey.clone(),
                    passphrase: params.passphrase,
                },
            };

            match request_context.admin_service.call(request).await {
                Ok(response) => match response {
                    AdminResponse::StartNode { macaroon } => {
                        let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                            .http_only(true)
                            .finish();
                        cookies.add(macaroon_cookie);
                        Ok(Json(json!({
                            "pubkey": node.pubkey,
                            "alias": node.alias,
                            "macaroon": macaroon,
                            "role": node.role
                        })))
                    }
                    AdminResponse::StartAdmin {
                        pubkey: _,
                        macaroon,
                        token,
                    } => {
                        let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                            .http_only(true)
                            .finish();
                        cookies.add(macaroon_cookie);
                        let token_cookie = Cookie::build("token", token).http_only(true).finish();
                        cookies.add(token_cookie);
                        Ok(Json(json!({
                            "pubkey": node.pubkey,
                            "alias": node.alias,
                            "macaroon": macaroon,
                            "role": node.role,
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
    Extension(request_context): Extension<Arc<RequestContext>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
) -> Result<Json<AdminResponse>, StatusCode> {
    let params: Result<CreateAdminParams, _> = serde_json::from_value(payload);
    let request = match params {
        Ok(params) => Ok(params.into()),
        Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }?;

    match request_context.admin_service.call(request).await {
        Ok(response) => match response {
            AdminResponse::CreateAdmin {
                pubkey,
                macaroon,
                external_id,
                role,
                token,
            } => {
                let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                    .http_only(true)
                    .finish();

                let token_cookie = Cookie::build("token", token.clone())
                    .http_only(true)
                    .finish();

                cookies.add(macaroon_cookie);
                cookies.add(token_cookie);
                Ok(Json(AdminResponse::CreateAdmin {
                    pubkey,
                    macaroon,
                    external_id,
                    role,
                    token,
                }))
            }
            _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
        },
        Err(err) => Ok(Json(AdminResponse::Error(err))),
    }
}

// this endpoint is overloaded and serves three purposes
// 1) is the root node created or not
// 2) is the node specified in my macaroon running?
// 3) is my macaroon valid?
pub async fn get_status(
    Extension(request_context): Extension<Arc<RequestContext>>,
    cookies: Cookies,
    AuthHeader { macaroon, token: _ }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let pubkey = {
        match get_macaroon_hex_str_from_cookies_or_header(&cookies, macaroon) {
            Ok(macaroon_hex) => match utils::macaroon_with_session_from_hex_str(&macaroon_hex) {
                Ok((_macaroon, session)) => session.pubkey,
                Err(_) => String::from(""),
            },
            Err(_) => String::from(""),
        }
    };

    match request_context
        .admin_service
        .call(AdminRequest::GetStatus { pubkey })
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(err) => Ok(Json(AdminResponse::Error(err))),
    }
}

pub async fn start_sensei(
    Extension(request_context): Extension<Arc<RequestContext>>,
    cookies: Cookies,
    Json(payload): Json<Value>,
) -> Result<Json<AdminResponse>, StatusCode> {
    let params: Result<StartAdminParams, _> = serde_json::from_value(payload);
    let request = match params {
        Ok(params) => Ok(params.into()),
        Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }?;

    match request {
        AdminRequest::StartAdmin { passphrase } => {
            match request_context
                .admin_service
                .call(AdminRequest::StartAdmin { passphrase })
                .await
            {
                Ok(response) => match response {
                    AdminResponse::StartAdmin {
                        pubkey,
                        macaroon,
                        token,
                    } => {
                        let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                            .http_only(true)
                            .permanent()
                            .finish();
                        cookies.add(macaroon_cookie);
                        let token_cookie = Cookie::build("token", token.clone())
                            .http_only(true)
                            .permanent()
                            .finish();
                        cookies.add(token_cookie);
                        Ok(Json(AdminResponse::StartAdmin {
                            pubkey,
                            macaroon,
                            token,
                        }))
                    }
                    _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
                },
                Err(_err) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
}

pub async fn create_node(
    Extension(request_context): Extension<Arc<RequestContext>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "nodes/create", &cookies, token).await?;
    let request = {
        let params: Result<CreateNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match request_context.admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn start_node(
    Extension(request_context): Extension<Arc<RequestContext>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "nodes/start", &cookies, token).await?;
    let request = {
        let params: Result<StartNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match request_context.admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn stop_node(
    Extension(request_context): Extension<Arc<RequestContext>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "nodes/stop", &cookies, token).await?;
    let request = {
        let params: Result<StopNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match request_context.admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn delete_node(
    Extension(request_context): Extension<Arc<RequestContext>>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    AuthHeader { macaroon: _, token }: AuthHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated =
        authenticate_request(&request_context, "nodes/delete", &cookies, token).await?;
    let request = {
        let params: Result<DeleteNodeParams, _> = serde_json::from_value(payload);
        match params {
            Ok(params) => Ok(params.into()),
            Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        }
    }?;

    if authenticated {
        match request_context.admin_service.call(request).await {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
