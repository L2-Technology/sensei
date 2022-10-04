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
        ConnectGossipPeerRequest, ConnectGossipPeerResponse, CreateAdminRequest,
        CreateAdminResponse, CreateNodeRequest, CreateNodeResponse, CreateTokenRequest,
        DeleteNodeRequest, DeleteNodeResponse, DeleteTokenRequest, DeleteTokenResponse,
        FindRouteRequest, FindRouteResponse, GetStatusRequest, GetStatusResponse, ListNode,
        ListNodesRequest, ListNodesResponse, ListTokensRequest, ListTokensResponse,
        NodeInfoRequest, NodeInfoResponse, PathFailedRequest, PathFailedResponse,
        PathSuccessfulRequest, PathSuccessfulResponse, Token,
    },
    utils::raw_macaroon_from_metadata,
};
use senseicore::{
    services::admin::{AdminRequest, AdminResponse},
    utils,
};
use tonic::{metadata::MetadataMap, Response, Status};

impl From<entity::access_token::Model> for Token {
    fn from(access_token: entity::access_token::Model) -> Token {
        Token {
            id: access_token.id,
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
            pagination: req.pagination.map(|p| p.into()).unwrap_or_default(),
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
                        id: node.id.clone(),
                        created_at: node.created_at,
                        updated_at: node.updated_at,
                        role: node.role as u32,
                        username: node.username,
                        alias: node.alias,
                        network: node.network,
                        listen_addr: node.listen_addr,
                        listen_port: node.listen_port as u32,
                        pubkey: node.id,
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
            pagination: req.pagination.map(|p| p.into()).unwrap_or_default(),
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
            entropy: req.entropy,
            cross_node_entropy: req.cross_node_entropy,
        }
    }
}

impl TryFrom<AdminResponse> for CreateNodeResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::CreateNode {
                pubkey,
                macaroon,
                listen_addr,
                listen_port,
                id,
                entropy,
                cross_node_entropy,
            } => Ok(Self {
                pubkey,
                macaroon,
                listen_addr,
                listen_port,
                id,
                entropy,
                cross_node_entropy,
            }),
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
            passphrase: req.passphrase,
        }
    }
}

impl TryFrom<AdminResponse> for CreateAdminResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::CreateAdmin { token } => Ok(Self { token }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl TryFrom<AdminResponse> for GetStatusResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::GetStatus {
                version,
                alias,
                setup,
                authenticated_node,
                authenticated_admin,
                pubkey,
                username,
                role,
            } => Ok(Self {
                version,
                alias,
                setup,
                authenticated_admin,
                authenticated_node,
                pubkey,
                username,
                role: role.map(|role| role as u32),
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

impl TryFrom<AdminResponse> for ConnectGossipPeerResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::ConnectGossipPeer {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<ConnectGossipPeerRequest> for AdminRequest {
    fn from(req: ConnectGossipPeerRequest) -> Self {
        AdminRequest::ConnectGossipPeer {
            node_connection_string: req.node_connection_string,
        }
    }
}

impl From<FindRouteRequest> for AdminRequest {
    fn from(req: FindRouteRequest) -> Self {
        AdminRequest::FindRoute {
            payer_public_key_hex: req.payer_public_key_hex,
            route_params_hex: req.route_params_hex,
            payment_hash_hex: req.payment_hash_hex,
            first_hops: req.first_hops,
        }
    }
}

impl TryFrom<AdminResponse> for FindRouteResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::FindRoute { route } => Ok(Self { route }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<NodeInfoRequest> for AdminRequest {
    fn from(req: NodeInfoRequest) -> Self {
        AdminRequest::NodeInfo {
            node_id_hex: req.node_id_hex,
        }
    }
}

impl TryFrom<AdminResponse> for NodeInfoResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::NodeInfo { node_info } => Ok(Self { node_info }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<PathSuccessfulRequest> for AdminRequest {
    fn from(req: PathSuccessfulRequest) -> Self {
        AdminRequest::PathSuccessful { path: req.path }
    }
}

impl TryFrom<AdminResponse> for PathSuccessfulResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::PathSuccessful {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<PathFailedRequest> for AdminRequest {
    fn from(req: PathFailedRequest) -> Self {
        AdminRequest::PathFailed {
            path: req.path,
            short_channel_id: req.short_channel_id,
        }
    }
}

impl TryFrom<AdminResponse> for PathFailedResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::PathFailed {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

pub struct AdminService {
    pub admin_service: Arc<senseicore::services::admin::AdminService>,
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
        let access_token = self
            .admin_service
            .database
            .get_access_token_by_token(token)
            .await;

        match access_token {
            Ok(Some(access_token)) => {
                if access_token.is_valid(scope) {
                    if access_token.single_use {
                        self.admin_service
                            .database
                            .delete_access_token(access_token.id)
                            .await
                            .unwrap();
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
    ) -> Result<AdminResponse, Status> {
        let required_scope = get_scope_from_request(&request);
        let token = self.raw_token_from_metadata(metadata)?;

        if self.is_valid_token(token, required_scope).await {
            self.admin_service
                .call(request)
                .await
                .map_err(|_e| Status::unknown("error"))
        } else {
            Err(Status::not_found("invalid or expired access token"))
        }
    }

    fn raw_token_from_metadata(&self, metadata: MetadataMap) -> Result<String, Status> {
        let token_opt = metadata.get("token");

        match token_opt {
            Some(token) => token
                .to_str()
                .map(String::from)
                .map_err(|_e| Status::unauthenticated("invalid token: must be ascii")),
            None => Err(Status::unauthenticated("token is required")),
        }
    }
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_status(
        &self,
        request: tonic::Request<GetStatusRequest>,
    ) -> Result<Response<GetStatusResponse>, Status> {
        let macaroon_hex_string = raw_macaroon_from_metadata(request.metadata().clone())?;
        let token = super::utils::raw_token_from_metadata(request.metadata().clone())?;

        let pubkey = match macaroon_hex_string {
            None => None,
            Some(macaroon_hex_string) => {
                let (_macaroon, session) =
                    utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
                        .map_err(|_e| Status::unauthenticated("invalid macaroon"))?;
                Some(session.pubkey)
            }
        };

        let authenticated_admin = match token {
            None => false,
            Some(token) => self.is_valid_token(token, Some("*")).await,
        };

        let request = AdminRequest::GetStatus {
            pubkey,
            authenticated_admin,
        };
        match self.admin_service.call(request).await {
            Ok(response) => {
                let response: Result<GetStatusResponse, String> = response.try_into();
                response
                    .map(Response::new)
                    .map_err(|_err| Status::unknown("err"))
            }
            Err(_err) => Err(Status::unknown("error")),
        }
    }
    async fn create_admin(
        &self,
        request: tonic::Request<CreateAdminRequest>,
    ) -> Result<Response<CreateAdminResponse>, Status> {
        let request: AdminRequest = request.into_inner().into();
        match self.admin_service.call(request).await {
            Ok(response) => {
                let response: Result<CreateAdminResponse, String> = response.try_into();
                response
                    .map(Response::new)
                    .map_err(|_err| Status::unknown("err"))
            }
            Err(_err) => Err(Status::unknown("error")),
        }
    }
    async fn start_node(
        &self,
        request: tonic::Request<AdminStartNodeRequest>,
    ) -> Result<Response<AdminStartNodeResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn stop_node(
        &self,
        request: tonic::Request<AdminStopNodeRequest>,
    ) -> Result<Response<AdminStopNodeResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_nodes(
        &self,
        request: tonic::Request<ListNodesRequest>,
    ) -> Result<Response<ListNodesResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_node(
        &self,
        request: tonic::Request<CreateNodeRequest>,
    ) -> Result<Response<CreateNodeResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn delete_node(
        &self,
        request: tonic::Request<DeleteNodeRequest>,
    ) -> Result<Response<DeleteNodeResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_tokens(
        &self,
        request: tonic::Request<ListTokensRequest>,
    ) -> Result<Response<ListTokensResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_token(
        &self,
        request: tonic::Request<CreateTokenRequest>,
    ) -> Result<Response<Token>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn delete_token(
        &self,
        request: tonic::Request<DeleteTokenRequest>,
    ) -> Result<Response<DeleteTokenResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn connect_gossip_peer(
        &self,
        request: tonic::Request<ConnectGossipPeerRequest>,
    ) -> Result<Response<ConnectGossipPeerResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn find_route(
        &self,
        request: tonic::Request<FindRouteRequest>,
    ) -> Result<Response<FindRouteResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn node_info(
        &self,
        request: tonic::Request<NodeInfoRequest>,
    ) -> Result<Response<NodeInfoResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn path_successful(
        &self,
        request: tonic::Request<PathSuccessfulRequest>,
    ) -> Result<Response<PathSuccessfulResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn path_failed(
        &self,
        request: tonic::Request<PathFailedRequest>,
    ) -> Result<Response<PathFailedResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
}
