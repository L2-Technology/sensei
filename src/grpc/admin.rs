// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

pub use super::sensei::admin_server::{Admin, AdminServer};
use super::sensei::{
    AdminStartNodeRequest, AdminStartNodeResponse, AdminStopNodeRequest, AdminStopNodeResponse,
    CreateAdminRequest, CreateAdminResponse, CreateNodeRequest, CreateNodeResponse,
    DeleteNodeRequest, DeleteNodeResponse, GetConfigRequest, GetConfigResponse, GetStatusRequest,
    GetStatusResponse, ListNode, ListNodesRequest, ListNodesResponse, StartAdminRequest,
    StartAdminResponse, UpdateConfigRequest, UpdateConfigResponse,
};
use crate::{
    services::admin::{AdminRequest, AdminResponse},
    utils,
};
use tonic::{metadata::MetadataMap, Response, Status};

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

impl From<CreateAdminRequest> for AdminRequest {
    fn from(req: CreateAdminRequest) -> Self {
        AdminRequest::CreateAdmin {
            username: req.username,
            alias: req.alias,
            passphrase: req.passphrase,
            electrum_url: req.electrum_url,
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
            } => Ok(Self {
                pubkey,
                macaroon,
                external_id,
                role: role as u32,
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
            AdminResponse::StartAdmin { pubkey, macaroon } => Ok(Self { pubkey, macaroon }),
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

impl From<GetConfigRequest> for AdminRequest {
    fn from(_req: GetConfigRequest) -> Self {
        AdminRequest::GetConfig {}
    }
}

impl TryFrom<AdminResponse> for GetConfigResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::GetConfig { electrum_url } => Ok(Self { electrum_url }),
            _ => Err("impossible".to_string()),
        }
    }
}

impl From<UpdateConfigRequest> for AdminRequest {
    fn from(req: UpdateConfigRequest) -> Self {
        AdminRequest::UpdateConfig {
            electrum_url: req.electrum_url,
        }
    }
}

impl TryFrom<AdminResponse> for UpdateConfigResponse {
    type Error = String;

    fn try_from(res: AdminResponse) -> Result<Self, Self::Error> {
        match res {
            AdminResponse::UpdateConfig {} => Ok(Self {}),
            _ => Err("impossible".to_string()),
        }
    }
}

pub struct AdminService {
    pub request_context: crate::RequestContext,
}

impl AdminService {
    async fn authenticated_request(
        &self,
        metadata: MetadataMap,
        request: AdminRequest,
    ) -> Result<AdminResponse, tonic::Status> {
        let macaroon_hex_string = self.raw_macaroon_from_metadata(metadata)?;

        let (macaroon, session) =
            utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
                .map_err(|_e| tonic::Status::unauthenticated("invalid macaroon"))?;
        let pubkey = session.pubkey.clone();

        let admin_node = {
            let mut admin_database = self.request_context.admin_service.database.lock().await;
            admin_database
                .get_admin_node()
                .map_err(|_e| tonic::Status::unknown("database error"))?
        };

        match admin_node {
            Some(node) => {
                if node.pubkey != pubkey {
                    return Err(Status::unauthenticated("invalid macaroon"));
                }

                let node_directory = self.request_context.node_directory.lock().await;

                match node_directory.get(&session.pubkey) {
                    Some(handle) => {
                        handle
                            .node
                            .verify_macaroon(macaroon, session)
                            .await
                            .map_err(|_e| {
                                Status::unauthenticated("invalid macaroon: failed to verify")
                            })?;

                        drop(node_directory);

                        self.request_context
                            .admin_service
                            .call(request)
                            .await
                            .map_err(|_e| Status::unknown("error"))
                    }
                    None => Err(Status::not_found("node with that pubkey not found")),
                }
            }
            None => Err(Status::not_found("admin node has not been created yet")),
        }
    }

    fn raw_macaroon_from_metadata(&self, metadata: MetadataMap) -> Result<String, tonic::Status> {
        let macaroon = metadata.get("macaroon");

        if macaroon.is_none() {
            return Err(Status::unauthenticated("macaroon is required"));
        }

        macaroon
            .unwrap()
            .to_str()
            .map(String::from)
            .map_err(|_e| Status::unauthenticated("invalid macaroon: must be ascii"))
    }
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_config(
        &self,
        request: tonic::Request<GetConfigRequest>,
    ) -> Result<tonic::Response<GetConfigResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn update_config(
        &self,
        request: tonic::Request<UpdateConfigRequest>,
    ) -> Result<tonic::Response<UpdateConfigResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn get_status(
        &self,
        request: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, tonic::Status> {
        let pubkey = {
            match self.raw_macaroon_from_metadata(request.metadata().clone()) {
                Ok(macaroon_hex_string) => {
                    match utils::macaroon_with_session_from_hex_str(&macaroon_hex_string) {
                        Ok((_macaroon, session)) => session.pubkey,
                        Err(_e) => String::from(""),
                    }
                }
                Err(_e) => String::from(""),
            }
        };

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
}
