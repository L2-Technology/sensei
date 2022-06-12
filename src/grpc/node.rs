// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::sync::Arc;

pub use super::sensei::node_server::{Node, NodeServer};

use super::{
    sensei::{
        CloseChannelRequest, CloseChannelResponse, ConnectPeerRequest, ConnectPeerResponse,
        CreateInvoiceRequest, CreateInvoiceResponse, DecodeInvoiceRequest, DecodeInvoiceResponse,
        DeletePaymentRequest, DeletePaymentResponse, GetBalanceRequest, GetBalanceResponse,
        GetUnusedAddressRequest, GetUnusedAddressResponse, InfoRequest, InfoResponse,
        KeysendRequest, KeysendResponse, LabelPaymentRequest, LabelPaymentResponse,
        ListChannelsRequest, ListChannelsResponse, ListPaymentsRequest, ListPaymentsResponse,
        ListPeersRequest, ListPeersResponse, ListUnspentRequest, ListUnspentResponse,
        OpenChannelsRequest, OpenChannelsResponse, PayInvoiceRequest, PayInvoiceResponse,
        SignMessageRequest, SignMessageResponse, StartNodeRequest, StartNodeResponse,
        StopNodeRequest, StopNodeResponse, VerifyMessageRequest, VerifyMessageResponse,
    },
    utils::raw_macaroon_from_metadata,
};

use senseicore::{
    services::{
        admin::AdminRequest,
        node::{NodeRequest, NodeResponse},
    },
    utils,
};
use tonic::{metadata::MetadataMap, Response, Status};

pub struct NodeService {
    pub admin_service: Arc<senseicore::services::admin::AdminService>,
}
impl NodeService {
    async fn authenticated_request(
        &self,
        metadata: MetadataMap,
        request: NodeRequest,
    ) -> Result<NodeResponse, tonic::Status> {
        let macaroon_hex_string = raw_macaroon_from_metadata(metadata)?;

        let (macaroon, session) =
            utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
                .map_err(|_e| tonic::Status::unauthenticated("invalid macaroon"))?;
        let pubkey = session.pubkey.clone();

        let node_directory = self.admin_service.node_directory.lock().await;

        match node_directory.get(&session.pubkey) {
            Some(Some(handle)) => {
                handle
                    .node
                    .verify_macaroon(macaroon, session)
                    .await
                    .map_err(|_e| Status::unauthenticated("invalid macaroon: failed to verify"))?;

                match request {
                    NodeRequest::StopNode {} => {
                        drop(node_directory);
                        let admin_request = AdminRequest::StopNode { pubkey };
                        let _ = self
                            .admin_service
                            .call(admin_request)
                            .await
                            .map_err(|_e| Status::unknown("failed to stop node"))?;
                        Ok(NodeResponse::StopNode {})
                    }
                    _ => handle
                        .node
                        .call(request)
                        .await
                        .map_err(|_e| Status::unknown("error")),
                }
            }
            Some(None) => Err(Status::not_found("node is in process of being started")),
            None => match request {
                NodeRequest::StartNode { passphrase } => {
                    drop(node_directory);
                    let admin_request = AdminRequest::StartNode {
                        passphrase,
                        pubkey: session.pubkey,
                    };
                    let _ = self.admin_service.call(admin_request).await.map_err(|_e| {
                        Status::unauthenticated("failed to start node, likely invalid passphrase")
                    })?;
                    Ok(NodeResponse::StartNode {})
                }
                _ => Err(Status::not_found("node with that pubkey not found")),
            },
        }
    }
}

#[tonic::async_trait]
impl Node for NodeService {
    async fn start_node(
        &self,
        request: tonic::Request<StartNodeRequest>,
    ) -> Result<tonic::Response<StartNodeResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn stop_node(
        &self,
        request: tonic::Request<StopNodeRequest>,
    ) -> Result<tonic::Response<StopNodeResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn get_unused_address(
        &self,
        request: tonic::Request<GetUnusedAddressRequest>,
    ) -> Result<tonic::Response<GetUnusedAddressResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn get_balance(
        &self,
        request: tonic::Request<GetBalanceRequest>,
    ) -> Result<tonic::Response<GetBalanceResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn open_channels(
        &self,
        request: tonic::Request<OpenChannelsRequest>,
    ) -> Result<tonic::Response<OpenChannelsResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn pay_invoice(
        &self,
        request: tonic::Request<PayInvoiceRequest>,
    ) -> Result<tonic::Response<PayInvoiceResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn decode_invoice(
        &self,
        request: tonic::Request<DecodeInvoiceRequest>,
    ) -> Result<tonic::Response<DecodeInvoiceResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn keysend(
        &self,
        request: tonic::Request<KeysendRequest>,
    ) -> Result<tonic::Response<KeysendResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_invoice(
        &self,
        request: tonic::Request<CreateInvoiceRequest>,
    ) -> Result<tonic::Response<CreateInvoiceResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn label_payment(
        &self,
        request: tonic::Request<LabelPaymentRequest>,
    ) -> Result<tonic::Response<LabelPaymentResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn delete_payment(
        &self,
        request: tonic::Request<DeletePaymentRequest>,
    ) -> Result<tonic::Response<DeletePaymentResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn connect_peer(
        &self,
        request: tonic::Request<ConnectPeerRequest>,
    ) -> Result<tonic::Response<ConnectPeerResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_channels(
        &self,
        request: tonic::Request<ListChannelsRequest>,
    ) -> Result<tonic::Response<ListChannelsResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_payments(
        &self,
        request: tonic::Request<ListPaymentsRequest>,
    ) -> Result<tonic::Response<ListPaymentsResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn close_channel(
        &self,
        request: tonic::Request<CloseChannelRequest>,
    ) -> Result<tonic::Response<CloseChannelResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn info(
        &self,
        request: tonic::Request<InfoRequest>,
    ) -> Result<tonic::Response<InfoResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_peers(
        &self,
        request: tonic::Request<ListPeersRequest>,
    ) -> Result<tonic::Response<ListPeersResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn sign_message(
        &self,
        request: tonic::Request<SignMessageRequest>,
    ) -> Result<tonic::Response<SignMessageResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn verify_message(
        &self,
        request: tonic::Request<VerifyMessageRequest>,
    ) -> Result<tonic::Response<VerifyMessageResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_unspent(
        &self,
        request: tonic::Request<ListUnspentRequest>,
    ) -> Result<tonic::Response<ListUnspentResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
}
