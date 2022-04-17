// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::sync::Arc;

pub use super::sensei::admin_server::{Admin, AdminServer};
use super::{
    sensei::{
        AdminStartNodeRequest, AdminStartNodeResponse, AdminStopNodeRequest, AdminStopNodeResponse,
        CreateAdminRequest, CreateAdminResponse, CreateNodeRequest, CreateNodeResponse,
        CreateTokenRequest, DeleteNodeRequest, DeleteNodeResponse, DeleteTokenRequest,
        DeleteTokenResponse, GetStatusRequest, GetStatusResponse, ListNode, ListNodesRequest,
        ListNodesResponse, ListTokensRequest, ListTokensResponse, StartAdminRequest,
        StartAdminResponse, Token,
    },
    utils::raw_macaroon_from_metadata,
};
use crate::{
    database::admin::AccessToken,
    services::admin::{AdminRequest, AdminResponse},
    utils,
};
use tonic::{metadata::MetadataMap, Response, Status};

impl From<AccessToken> for Token {
    fn from(access_token: AccessToken) -> Token {
        Token {
            id: access_token.id,
            external_id: access_token.external_id,
            created_at: access_token.created_at,
            updated_at: access_token.updated_at,
            expires_at: access_token.expires_at,
            token: access_token.token,
            name: access_token.name,
            single_use: access_token.single_use,
            scope: access_token.scope,
        }
    }
}

impl From<ListNodesRequest> for AdminRequest {
    fn from(req: ListNodesRequest) -> Self {
        AdminRequest::ListNodes {
            pagination: req.pagination.into(),
        }
    }
}

impl TryFrom<AdminResponse> for ListNodesResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::ListNodes { nodes, pagination } => Ok(Self {
                nodes: nodes
                    .into_iter()
                    .map(|node| ListNode {
                        id: node.id,
                        external_id: node.external_id,
                        created_at: node.created_at,
                        updated_at: node.updated_at,
                        role: node.role as u32,
                        username: node.username,
                        alias: node.alias,
                        network: node.network,
                        listen_addr: node.listen_addr,
                        listen_port: node.listen_port as u32,
                        pubkey: node.pubkey,
                        status: node.status as u32,
                    })
                    .collect::<Vec<ListNode>>(),
                pagination: Some(pagination.into()),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ListTokensRequest> for AdminRequest {
    fn from(req: ListTokensRequest) -> Self {
        AdminRequest::ListTokens {
            pagination: req.pagination.into(),
        }
    }
}

impl TryFrom<AdminResponse> for ListTokensResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::ListTokens { tokens, pagination } => Ok(Self {
                tokens: tokens
                    .into_iter()
                    .map(|token| token.into())
                    .collect::<Vec<Token>>(),
                pagination: Some(pagination.into()),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<CreateNodeRequest> for AdminRequest {
    fn from(req: CreateNodeRequest) -> Self {
        AdminRequest::CreateNode {
            username: req.username,
            alias: req.alias,
            passphrase: req.passphrase,
            start: req.start,
        }
    }
}

impl TryFrom<AdminResponse> for CreateNodeResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::CreateNode { pubkey, macaroon } => Ok(Self { pubkey, macaroon }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<CreateTokenRequest> for AdminRequest {
    fn from(req: CreateTokenRequest) -> Self {
        AdminRequest::CreateToken {
            name: req.name,
            scope: req.scope,
            expires_at: req.expires_at,
            single_use: req.single_use,
        }
    }
}

impl TryFrom<AdminResponse> for Token {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::CreateToken { token } => Ok(token.into()),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<CreateAdminRequest> for AdminRequest {
    fn from(req: CreateAdminRequest) -> Self {
        AdminRequest::CreateAdmin {
            username: req.username,
            alias: req.alias,
            passphrase: req.passphrase,
            start: req.start,
        }
    }
}

impl TryFrom<AdminResponse> for CreateAdminResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::CreateAdmin {
                pubkey,
                macaroon,
                external_id,
                role,
                token,
            } => Ok(Self {
                pubkey,
                macaroon,
                external_id,
                role: role as u32,
                token,
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl TryFrom<AdminResponse> for GetStatusResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::GetStatus {
                alias,
                running,
                created,
                authenticated,
                pubkey,
                username,
                role,
            } => Ok(Self {
                alias,
                running,
                created,
                authenticated,
                pubkey,
                username,
                role: role.map(|role| role as u32),
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<StartAdminRequest> for AdminRequest {
    fn from(req: StartAdminRequest) -> Self {
        AdminRequest::StartAdmin {
            passphrase: req.passphrase,
        }
    }
}

impl TryFrom<AdminResponse> for StartAdminResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::StartAdmin {
                pubkey,
                macaroon,
                token,
            } => Ok(Self {
                pubkey,
                macaroon,
                token,
            }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<AdminStartNodeRequest> for AdminRequest {
    fn from(req: AdminStartNodeRequest) -> Self {
        AdminRequest::StartNode {
            passphrase: req.passphrase,
            pubkey: req.pubkey,
        }
    }
}

impl TryFrom<AdminResponse> for AdminStartNodeResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::StartNode { macaroon } => Ok(Self { macaroon }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<AdminStopNodeRequest> for AdminRequest {
    fn from(req: AdminStopNodeRequest) -> Self {
        AdminRequest::StopNode { pubkey: req.pubkey }
    }
}

impl TryFrom<AdminResponse> for AdminStopNodeResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::StopNode {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<DeleteNodeRequest> for AdminRequest {
    fn from(req: DeleteNodeRequest) -> Self {
        AdminRequest::DeleteNode { pubkey: req.pubkey }
    }
}

impl TryFrom<AdminResponse> for DeleteNodeResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::DeleteNode {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<DeleteTokenRequest> for AdminRequest {
    fn from(req: DeleteTokenRequest) -> Self {
        AdminRequest::DeleteToken { id: req.id }
    }
}

impl TryFrom<AdminResponse> for DeleteTokenResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::DeleteToken {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}
pub struct AdminService {
    pub request_context: Arc<crate::RequestContext>,
}

pub fn get_scope_from_request(request: &AdminRequest) -> Option<&'static str> {
    match request {
        AdminRequest::CreateNode { .. } => Some("nodes/create"),
        AdminRequest::ListNodes { .. } => Some("nodes/list"),
        AdminRequest::DeleteNode { .. } => Some("nodes/delete"),
        AdminRequest::StopNode { .. } => Some("nodes/stop"),
        AdminRequest::ListTokens { .. } => Some("tokens/list"),
        AdminRequest::CreateToken { .. } => Some("tokens/create"),
        AdminRequest::DeleteToken { .. } => Some("tokens/delete"),
        _ => None,
    }
}

impl AdminService {
    async fn is_valid_token(&self, token: String, scope: Option<&str>) -> bool {
        let access_token = {
            let mut admin_database = self.request_context.admin_service.database.lock().await;
            admin_database.get_access_token(token)
        };

        match access_token {
            Ok(Some(access_token)) => {
                if access_token.is_valid(scope) {
                    if access_token.single_use {
                        let mut database = self.request_context.admin_service.database.lock().await;
                        database.delete_access_token(access_token.id).unwrap();
                    }
                    true
                } else {
                    false
                }
            }
            Ok(None) => false,
            Err(_) => false,
        }
    }

    async fn authenticated_request(
        &self,
        metadata: MetadataMap,
        request: AdminRequest,
    ) -> Result<AdminResponse, tonic::Status> {
        let required_scope = get_scope_from_request(&request);

        let token = self.raw_token_from_metadata(metadata)?;

        if self.is_valid_token(token, required_scope).await {
            self.request_context
                .admin_service
                .call(request)
                .await
                .map_err(|_e| Status::unknown("error"))
        } else {
            Err(Status::not_found("invalid or expired access token"))
        }
    }

    fn raw_token_from_metadata(&self, metadata: MetadataMap) -> Result<String, tonic::Status> {
        let token = metadata.get("token");

        if token.is_none() {
            return Err(Status::unauthenticated("token is required"));
        }

        token
            .unwrap()
            .to_str()
            .map(String::from)
            .map_err(|_e| Status::unauthenticated("invalid token: must be ascii"))
    }
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_status(
        &self,
        request: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, tonic::Status> {
        let macaroon_hex_string = raw_macaroon_from_metadata(request.metadata().clone())?;

        let (_macaroon, session) = utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
            .map_err(|_e| tonic::Status::unauthenticated("invalid macaroon"))?;
        let pubkey = session.pubkey.clone();

        let request = AdminRequest::GetStatus { pubkey };
        match self.request_context.admin_service.call(request).await {
            Ok(response) => {
                let response: Result<GetStatusResponse, String> = response.try_into();
                response
                    .map(Response::new)
                    .map_err(|_err| tonic::Status::unknown("err"))
            }
            Err(_err) => Err(tonic::Status::unknown("error")),
        }
    }
    async fn create_admin(
        &self,
        request: tonic::Request<CreateAdminRequest>,
    ) -> Result<tonic::Response<CreateAdminResponse>, tonic::Status> {
        let request: AdminRequest = request.into_inner().into();
        match self.request_context.admin_service.call(request).await {
            Ok(response) => {
                let response: Result<CreateAdminResponse, String> = response.try_into();
                response
                    .map(Response::new)
                    .map_err(|_err| tonic::Status::unknown("err"))
            }
            Err(_err) => Err(tonic::Status::unknown("error")),
        }
    }
    async fn start_admin(
        &self,
        request: tonic::Request<StartAdminRequest>,
    ) -> Result<tonic::Response<StartAdminResponse>, tonic::Status> {
        let request: AdminRequest = request.into_inner().into();
        match self.request_context.admin_service.call(request).await {
            Ok(response) => {
                let response: Result<StartAdminResponse, String> = response.try_into();
                response
                    .map(Response::new)
                    .map_err(|_err| tonic::Status::unknown("err"))
            }
            Err(_err) => Err(tonic::Status::unknown("error")),
        }
    }
    async fn start_node(
        &self,
        request: tonic::Request<AdminStartNodeRequest>,
    ) -> Result<tonic::Response<AdminStartNodeResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn stop_node(
        &self,
        request: tonic::Request<AdminStopNodeRequest>,
    ) -> Result<tonic::Response<AdminStopNodeResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_nodes(
        &self,
        request: tonic::Request<ListNodesRequest>,
    ) -> Result<tonic::Response<ListNodesResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_node(
        &self,
        request: tonic::Request<CreateNodeRequest>,
    ) -> Result<tonic::Response<CreateNodeResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn delete_node(
        &self,
        request: tonic::Request<DeleteNodeRequest>,
    ) -> Result<tonic::Response<DeleteNodeResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_tokens(
        &self,
        request: tonic::Request<ListTokensRequest>,
    ) -> Result<tonic::Response<ListTokensResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_token(
        &self,
        request: tonic::Request<CreateTokenRequest>,
    ) -> Result<tonic::Response<Token>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn delete_token(
        &self,
        request: tonic::Request<DeleteTokenRequest>,
    ) -> Result<tonic::Response<DeleteTokenResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
}
