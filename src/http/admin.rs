// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use axum::{
    extract::{Extension, Query},
    routing::{get, post},
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

use super::macaroon_header::MacaroonHeader;

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
pub struct UpdateConfigParams {
    pub electrum_url: String,
}

impl From<UpdateConfigParams> for AdminRequest {
    fn from(params: UpdateConfigParams) -> Self {
        Self::UpdateConfig {
            electrum_url: params.electrum_url,
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
pub struct CreateAdminParams {
    pub username: String,
    pub passphrase: String,
    pub alias: String,
    pub electrum_url: String,
    pub start: bool,
}

impl From<CreateAdminParams> for AdminRequest {
    fn from(params: CreateAdminParams) -> Self {
        Self::CreateAdmin {
            username: params.username,
            passphrase: params.passphrase,
            alias: params.alias,
            electrum_url: params.electrum_url,
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

pub fn get_macaroon_hex_str_from_cookies_or_header(
    cookies: &Cookies,
    macaroon: Option<HeaderValue>,
) -> Result<String, StatusCode> {
    match macaroon {
        Some(macaroon) => {
            let res = macaroon
                .to_str()
                .map(|str| str.to_string())
                .map_err(|_| StatusCode::UNAUTHORIZED);
            res
        }
        None => match cookies.get("macaroon") {
            Some(macaroon_cookie) => {
                let macaroon_cookie_str = macaroon_cookie.value().to_string();
                Ok(macaroon_cookie_str)
            }
            None => Err(StatusCode::UNAUTHORIZED),
        },
    }
}

pub async fn authenticate_request(
    request_context: &RequestContext,
    cookies: &Cookies,
    macaroon: Option<HeaderValue>,
) -> Result<bool, StatusCode> {
    let macaroon_hex_string = get_macaroon_hex_str_from_cookies_or_header(cookies, macaroon)?;

    let (macaroon, session) = utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
        .map_err(|_e| StatusCode::UNAUTHORIZED)?;

    let pubkey = session.pubkey.clone();

    let admin_node = {
        let mut admin_database = request_context.admin_service.database.lock().await;
        admin_database
            .get_admin_node()
            .map_err(|_e| StatusCode::UNAUTHORIZED)?
    };

    match admin_node {
        Some(admin_node) => {
            if admin_node.pubkey != pubkey {
                return Ok(false);
            }

            let node_directory = request_context.node_directory.lock().await;
            let node = node_directory.get(&session.pubkey);

            match node {
                Some(handle) => {
                    handle
                        .node
                        .verify_macaroon(macaroon, session)
                        .await
                        .map_err(|_e| StatusCode::UNAUTHORIZED)?;

                    Ok(true)
                }
                None => Ok(false),
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
        .route("/v1/status", get(get_status))
        .route("/v1/start", post(start_sensei))
        .route("/v1/login", post(login))
        .route("/v1/logout", post(logout))
        .route("/v1/config", get(get_config))
        .route("/v1/config", post(update_config))
}

pub async fn get_config(
    Extension(request_context): Extension<RequestContext>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
    if authenticated {
        match request_context
            .admin_service
            .call(AdminRequest::GetConfig {})
            .await
        {
            Ok(response) => Ok(Json(response)),
            Err(_err) => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn update_config(
    Extension(request_context): Extension<RequestContext>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
    let request = {
        let params: Result<UpdateConfigParams, _> = serde_json::from_value(payload);
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
    Extension(request_context): Extension<RequestContext>,
    cookies: Cookies,
    Query(pagination): Query<PaginationRequest>,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
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
    Extension(request_context): Extension<RequestContext>,
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
            let request = AdminRequest::StartNode {
                pubkey: node.pubkey.clone(),
                passphrase: params.passphrase,
            };

            match request_context.admin_service.call(request).await {
                Ok(response) => match response {
                    AdminResponse::StartNode { macaroon } => {
                        let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                            .domain("localhost")
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
    Ok(Json::default())
}

pub async fn init_sensei(
    Extension(request_context): Extension<RequestContext>,
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
            } => {
                let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                    .domain("localhost")
                    .http_only(true)
                    .finish();
                cookies.add(macaroon_cookie);
                Ok(Json(AdminResponse::CreateAdmin {
                    pubkey,
                    macaroon,
                    external_id,
                    role,
                }))
            }
            _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
        },
        Err(err) => Ok(Json(AdminResponse::Error(err))),
    }
}

pub async fn get_status(
    Extension(request_context): Extension<RequestContext>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let pubkey = {
        match get_macaroon_hex_str_from_cookies_or_header(&cookies, macaroon) {
            Ok(macaroon_hex_string) => {
                match utils::macaroon_with_session_from_hex_str(&macaroon_hex_string) {
                    Ok((_macaroon, session)) => session.pubkey,
                    Err(_e) => String::from(""),
                }
            }
            Err(_e) => String::from(""),
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
    Extension(request_context): Extension<RequestContext>,
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
                    AdminResponse::StartAdmin { pubkey, macaroon } => {
                        let macaroon_cookie = Cookie::build("macaroon", macaroon.clone())
                            .domain("localhost")
                            .http_only(true)
                            .permanent()
                            .finish();
                        cookies.add(macaroon_cookie);
                        Ok(Json(AdminResponse::StartAdmin { pubkey, macaroon }))
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
    Extension(request_context): Extension<RequestContext>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
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
    Extension(request_context): Extension<RequestContext>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
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
    Extension(request_context): Extension<RequestContext>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
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
    Extension(request_context): Extension<RequestContext>,
    Json(payload): Json<Value>,
    cookies: Cookies,
    MacaroonHeader(macaroon): MacaroonHeader,
) -> Result<Json<AdminResponse>, StatusCode> {
    let authenticated = authenticate_request(&request_context, &cookies, macaroon).await?;
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
