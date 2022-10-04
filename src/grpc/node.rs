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
        AddKnownPeerRequest, AddKnownPeerResponse, CloseChannelRequest, CloseChannelResponse,
        ConnectPeerRequest, ConnectPeerResponse, CreateInvoiceRequest, CreateInvoiceResponse,
        CreatePhantomInvoiceRequest, CreatePhantomInvoiceResponse, DecodeInvoiceRequest,
        DecodeInvoiceResponse, DeletePaymentRequest, DeletePaymentResponse, GetBalanceRequest,
        GetBalanceResponse, GetPhantomRouteHintsRequest, GetPhantomRouteHintsResponse,
        GetUnusedAddressRequest, GetUnusedAddressResponse, InfoRequest, InfoResponse,
        KeysendRequest, KeysendResponse, LabelPaymentRequest, LabelPaymentResponse,
        ListChannelsRequest, ListChannelsResponse, ListKnownPeersRequest, ListKnownPeersResponse,
        ListPaymentsRequest, ListPaymentsResponse, ListPeersRequest, ListPeersResponse,
        ListPhantomPaymentsRequest, ListPhantomPaymentsResponse, ListUnspentRequest,
        ListUnspentResponse, NetworkGraphInfoRequest, NetworkGraphInfoResponse,
        OpenChannelsRequest, OpenChannelsResponse, PayInvoiceRequest, PayInvoiceResponse,
        RemoveKnownPeerRequest, RemoveKnownPeerResponse, SignMessageRequest, SignMessageResponse,
        StartNodeRequest, StartNodeResponse, StopNodeRequest, StopNodeResponse,
        VerifyMessageRequest, VerifyMessageResponse,
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
    ) -> Result<NodeResponse, Status> {
        match raw_macaroon_from_metadata(metadata)? {
            None => Err(Status::unauthenticated("macaroon required")),
            Some(macaroon_hex_string) => {
                let (macaroon, session) =
                    utils::macaroon_with_session_from_hex_str(&macaroon_hex_string)
                        .map_err(|_e| Status::unauthenticated("invalid macaroon"))?;
                let pubkey = session.pubkey.clone();

                let node_directory = self.admin_service.node_directory.lock().await;

                match node_directory.get(&session.pubkey) {
                    Some(Some(handle)) => {
                        handle
                            .node
                            .verify_macaroon(macaroon, session)
                            .await
                            .map_err(|_e| {
                                Status::unauthenticated("invalid macaroon: failed to verify")
                            })?;

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
                                Status::unauthenticated(
                                    "failed to start node, likely invalid passphrase",
                                )
                            })?;
                            Ok(NodeResponse::StartNode {})
                        }
                        _ => Err(Status::not_found("node with that pubkey not found")),
                    },
                }
            }
        }
    }
}

#[tonic::async_trait]
impl Node for NodeService {
    async fn start_node(
        &self,
        request: tonic::Request<StartNodeRequest>,
    ) -> Result<Response<StartNodeResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn stop_node(
        &self,
        request: tonic::Request<StopNodeRequest>,
    ) -> Result<Response<StopNodeResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn get_unused_address(
        &self,
        request: tonic::Request<GetUnusedAddressRequest>,
    ) -> Result<Response<GetUnusedAddressResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }

    async fn get_balance(
        &self,
        request: tonic::Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn open_channels(
        &self,
        request: tonic::Request<OpenChannelsRequest>,
    ) -> Result<Response<OpenChannelsResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn pay_invoice(
        &self,
        request: tonic::Request<PayInvoiceRequest>,
    ) -> Result<Response<PayInvoiceResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn decode_invoice(
        &self,
        request: tonic::Request<DecodeInvoiceRequest>,
    ) -> Result<Response<DecodeInvoiceResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn keysend(
        &self,
        request: tonic::Request<KeysendRequest>,
    ) -> Result<Response<KeysendResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_invoice(
        &self,
        request: tonic::Request<CreateInvoiceRequest>,
    ) -> Result<Response<CreateInvoiceResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn create_phantom_invoice(
        &self,
        request: tonic::Request<CreatePhantomInvoiceRequest>,
    ) -> Result<Response<CreatePhantomInvoiceResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn get_phantom_route_hints(
        &self,
        request: tonic::Request<GetPhantomRouteHintsRequest>,
    ) -> Result<Response<GetPhantomRouteHintsResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn label_payment(
        &self,
        request: tonic::Request<LabelPaymentRequest>,
    ) -> Result<Response<LabelPaymentResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn delete_payment(
        &self,
        request: tonic::Request<DeletePaymentRequest>,
    ) -> Result<Response<DeletePaymentResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn connect_peer(
        &self,
        request: tonic::Request<ConnectPeerRequest>,
    ) -> Result<Response<ConnectPeerResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_channels(
        &self,
        request: tonic::Request<ListChannelsRequest>,
    ) -> Result<Response<ListChannelsResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_payments(
        &self,
        request: tonic::Request<ListPaymentsRequest>,
    ) -> Result<Response<ListPaymentsResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_phantom_payments(
        &self,
        request: tonic::Request<ListPhantomPaymentsRequest>,
    ) -> Result<Response<ListPhantomPaymentsResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn close_channel(
        &self,
        request: tonic::Request<CloseChannelRequest>,
    ) -> Result<Response<CloseChannelResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn info(
        &self,
        request: tonic::Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_peers(
        &self,
        request: tonic::Request<ListPeersRequest>,
    ) -> Result<Response<ListPeersResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn sign_message(
        &self,
        request: tonic::Request<SignMessageRequest>,
    ) -> Result<Response<SignMessageResponse>, Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn verify_message(
        &self,
        request: tonic::Request<VerifyMessageRequest>,
    ) -> Result<Response<VerifyMessageResponse>, Status> {
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
    async fn network_graph_info(
        &self,
        request: tonic::Request<NetworkGraphInfoRequest>,
    ) -> Result<tonic::Response<NetworkGraphInfoResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn list_known_peers(
        &self,
        request: tonic::Request<ListKnownPeersRequest>,
    ) -> Result<tonic::Response<ListKnownPeersResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn add_known_peer(
        &self,
        request: tonic::Request<AddKnownPeerRequest>,
    ) -> Result<tonic::Response<AddKnownPeerResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
    async fn remove_known_peer(
        &self,
        request: tonic::Request<RemoveKnownPeerRequest>,
    ) -> Result<tonic::Response<RemoveKnownPeerResponse>, tonic::Status> {
        self.authenticated_request(request.metadata().clone(), request.into_inner().into())
            .await?
            .try_into()
            .map(Response::new)
            .map_err(|_e| Status::unknown("unknown error"))
    }
}
