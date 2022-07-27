use bitcoin::secp256k1::PublicKey;
use lightning::ln::channelmanager::ChannelDetails;
use lightning::ln::msgs::{ErrorAction, LightningError};
use lightning::ln::PaymentHash;
use lightning::routing::gossip::NodeId;
use lightning::routing::router::{Route, RouteHop, RouteParameters};
use lightning::routing::scoring::{ChannelUsage, Score};
use lightning::util::ser::{Readable, Writeable};
use lightning_invoice::payment::Router as LdkRouterTrait;
use serde::Deserialize;
use std::io::Cursor;
use tokio::runtime::Handle;

use crate::hex_utils;
use crate::node::{Router, Scorer};

pub enum AnyRouter {
    Local(Router),
    Remote(RemoteRouter),
}

impl AnyRouter {
    pub fn new_remote(host: String, token: String, handle: Handle) -> Self {
        AnyRouter::Remote(RemoteRouter::new(host, token, handle))
    }
}

impl<S: Score> LdkRouterTrait<S> for AnyRouter {
    fn find_route(
        &self,
        payer: &PublicKey,
        route_params: &RouteParameters,
        payment_hash: &PaymentHash,
        first_hops: Option<&[&ChannelDetails]>,
        scorer: &S,
    ) -> Result<Route, LightningError> {
        match self {
            AnyRouter::Local(router) => {
                router.find_route(payer, route_params, payment_hash, first_hops, scorer)
            }
            AnyRouter::Remote(router) => {
                router.find_route(payer, route_params, payment_hash, first_hops, scorer)
            }
        }
    }
}

pub enum AnyScorer {
    Local(Scorer),
    Remote(RemoteScorer),
}

impl AnyScorer {
    pub fn new_remote(host: String, token: String, handle: Handle) -> Self {
        AnyScorer::Remote(RemoteScorer::new(host, token, handle))
    }
}

impl Score for AnyScorer {
    fn channel_penalty_msat(
        &self,
        short_channel_id: u64,
        source: &NodeId,
        target: &NodeId,
        usage: ChannelUsage,
    ) -> u64 {
        match self {
            AnyScorer::Local(scorer) => {
                scorer.channel_penalty_msat(short_channel_id, source, target, usage)
            }
            AnyScorer::Remote(scorer) => {
                scorer.channel_penalty_msat(short_channel_id, source, target, usage)
            }
        }
    }

    fn payment_path_failed(&mut self, path: &[&RouteHop], short_channel_id: u64) {
        match self {
            AnyScorer::Local(scorer) => scorer.payment_path_failed(path, short_channel_id),
            AnyScorer::Remote(scorer) => scorer.payment_path_failed(path, short_channel_id),
        }
    }

    fn payment_path_successful(&mut self, path: &[&RouteHop]) {
        match self {
            AnyScorer::Local(scorer) => scorer.payment_path_successful(path),
            AnyScorer::Remote(scorer) => scorer.payment_path_successful(path),
        }
    }

    fn probe_failed(&mut self, path: &[&RouteHop], short_channel_id: u64) {
        match self {
            AnyScorer::Local(scorer) => scorer.probe_failed(path, short_channel_id),
            AnyScorer::Remote(scorer) => scorer.probe_failed(path, short_channel_id),
        }
    }

    fn probe_successful(&mut self, path: &[&RouteHop]) {
        match self {
            AnyScorer::Local(scorer) => scorer.probe_successful(path),
            AnyScorer::Remote(scorer) => scorer.probe_successful(path),
        }
    }
}

impl Writeable for AnyScorer {
    fn write<W: lightning::util::ser::Writer>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        match self {
            AnyScorer::Local(scorer) => writer.write_all(&scorer.encode()),
            AnyScorer::Remote(_scorer) => writer.write_all(&[0]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RemoteSenseiInfo {
    pub host: String,
    pub token: String,
}

#[derive(Clone)]
pub struct RemoteScorer {
    remote_sensei: RemoteSenseiInfo,
    tokio_handle: Handle,
}

impl RemoteScorer {
    pub fn new(host: String, token: String, tokio_handle: Handle) -> Self {
        Self {
            remote_sensei: RemoteSenseiInfo { host, token },
            tokio_handle,
        }
    }
    fn payment_path_failed_route(&self) -> String {
        format!("{}/v1/ldk/network/path/failed", self.remote_sensei.host)
    }
    fn payment_path_successful_route(&self) -> String {
        format!("{}/v1/ldk/network/path/successful", self.remote_sensei.host)
    }

    async fn payment_path_failed_async(&mut self, path: &[&RouteHop], short_channel_id: u64) {
        let client = reqwest::Client::new();
        let _res = client.post(self.payment_path_failed_route())
          .header("token", self.remote_sensei.token.clone())
          .json(&serde_json::json!({
            "path": path.iter().map(|route_hop| { hex_utils::hex_str(&route_hop.encode()) }).collect::<Vec<_>>(),
            "short_channel_id": short_channel_id
          }))
          .send().await;
    }

    async fn payment_path_successful_async(&mut self, path: &[&RouteHop]) {
        let client = reqwest::Client::new();
        let _res = client.post(self.payment_path_successful_route())
          .header("token", self.remote_sensei.token.clone())
          .json(&serde_json::json!({
            "path": path.iter().map(|route_hop| { hex_utils::hex_str(&route_hop.encode()) }).collect::<Vec<_>>()
          }))
          .send()
          .await;
    }
}

impl Score for RemoteScorer {
    fn channel_penalty_msat(
        &self,
        _short_channel_id: u64,
        _source: &NodeId,
        _target: &NodeId,
        _usage: ChannelUsage,
    ) -> u64 {
        // unreachable
        // when using RemoteScorer it means you are using a RemoteRouter
        // this is only called during find_route which happens on the remote, not here.
        // but we need to return something
        0
    }

    fn payment_path_failed(&mut self, path: &[&RouteHop], short_channel_id: u64) {
        tokio::task::block_in_place(move || {
            self.tokio_handle.clone().block_on(async move {
                self.payment_path_failed_async(path, short_channel_id).await;
            })
        })
    }

    fn payment_path_successful(&mut self, path: &[&RouteHop]) {
        tokio::task::block_in_place(move || {
            self.tokio_handle.clone().block_on(async move {
                self.payment_path_successful_async(path).await;
            })
        })
    }

    fn probe_failed(&mut self, path: &[&RouteHop], short_channel_id: u64) {
        self.payment_path_failed(path, short_channel_id);
    }

    fn probe_successful(&mut self, path: &[&RouteHop]) {
        self.payment_path_successful(path);
    }
}

#[derive(Deserialize)]
struct FindRouteResponse {
    route: String,
}

#[derive(Clone)]
pub struct RemoteRouter {
    pub remote_sensei: RemoteSenseiInfo,
    pub tokio_handle: Handle,
}

impl RemoteRouter {
    pub fn new(host: String, token: String, tokio_handle: Handle) -> Self {
        Self {
            remote_sensei: RemoteSenseiInfo { host, token },
            tokio_handle,
        }
    }
    fn find_route_route(&self) -> String {
        format!("{}/v1/ldk/network/route", self.remote_sensei.host)
    }

    async fn find_route_async(
        &self,
        payer: &PublicKey,
        route_params: &RouteParameters,
        payment_hash: &PaymentHash,
        first_hops: Option<&[&ChannelDetails]>,
    ) -> Result<Route, LightningError> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.find_route_route())
            .header("token", self.remote_sensei.token.clone())
            .json(&serde_json::json!({
                "payer_public_key_hex": hex_utils::hex_str(&payer.encode()),
                "route_params_hex": hex_utils::hex_str(&route_params.encode()),
                "payment_hash_hex": hex_utils::hex_str(&payment_hash.encode()),
                "first_hops": first_hops.unwrap_or_default().iter().map(|hop| {
                hex_utils::hex_str(&hop.encode())
                }).collect::<Vec<_>>(),
            }))
            .send()
            .await
            .map_err(|error| LightningError {
                err: error.to_string(),
                action: ErrorAction::IgnoreError,
            });

        match response {
            Ok(response) => {
                let find_route_response: FindRouteResponse = response.json().await.unwrap();
                let mut readable_route =
                    Cursor::new(hex_utils::to_vec(&find_route_response.route).unwrap());
                Ok(Route::read(&mut readable_route).unwrap())
            }
            Err(e) => Err(e),
        }
    }
}

impl<S: Score> LdkRouterTrait<S> for RemoteRouter {
    fn find_route(
        &self,
        payer: &PublicKey,
        route_params: &RouteParameters,
        payment_hash: &PaymentHash,
        first_hops: Option<&[&ChannelDetails]>,
        _scorer: &S,
    ) -> Result<Route, LightningError> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.clone().block_on(async move {
                self.find_route_async(payer, route_params, payment_hash, first_hops)
                    .await
            })
        })
    }
}
