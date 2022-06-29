use std::io::Cursor;

use bitcoin::secp256k1::PublicKey;
use lightning::ln::channelmanager::ChannelDetails;
use lightning::ln::msgs::{ErrorAction, LightningError};
use lightning::ln::PaymentHash;
use lightning::routing::gossip::NodeId;
use lightning::routing::router::{Route, RouteHop, RouteParameters};
use lightning::routing::scoring::{ChannelUsage, Score};
use lightning::util::ser::{Readable, Writeable};
use lightning_invoice::payment::Router;
use serde::Deserialize;

use crate::hex_utils;

#[derive(Clone, Debug)]
pub struct RemoteSenseiInfo {
    pub host: String,
    pub token: String,
}

#[derive(Clone)]
pub struct RemoteScorer {
    remote_sensei: RemoteSenseiInfo,
}

impl RemoteScorer {
    fn payment_path_failed_route(&self) -> String {
        format!("{}/v1/ldk/network/path/failed", self.remote_sensei.host)
    }
    fn payment_path_successful_route(&self) -> String {
        format!("{}/v1/ldk/network/path/successful", self.remote_sensei.host)
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
        let client = reqwest::blocking::Client::new();
        let _res = client.post(self.payment_path_failed_route())
          .header("token", self.remote_sensei.token.clone())
          .json(&serde_json::json!({
            "path": path.iter().map(|route_hop| { hex_utils::hex_str(&route_hop.encode()) }).collect::<Vec<_>>(),
            "short_channel_id": short_channel_id
          }))
          .send();
    }

    fn payment_path_successful(&mut self, path: &[&RouteHop]) {
        let client = reqwest::blocking::Client::new();
        let _res = client.post(self.payment_path_successful_route())
          .header("token", self.remote_sensei.token.clone())
          .json(&serde_json::json!({
            "path": path.iter().map(|route_hop| { hex_utils::hex_str(&route_hop.encode()) }).collect::<Vec<_>>()
          }))
          .send();
    }
}

#[derive(Deserialize)]
struct FindRouteResponse {
    route: String,
}

#[derive(Clone)]
struct RemoteRouter {
    remote_sensei: RemoteSenseiInfo,
}

impl RemoteRouter {
    fn find_route_route(&self) -> String {
        format!("{}/v1/ldk/network/route", self.remote_sensei.host)
    }
}

impl<S: Score> Router<S> for RemoteRouter {
    fn find_route(
        &self,
        payer: &PublicKey,
        route_params: &RouteParameters,
        payment_hash: &PaymentHash,
        first_hops: Option<&[&ChannelDetails]>,
        _scorer: &S,
    ) -> Result<Route, LightningError> {
        let client = reqwest::blocking::Client::new();
        client
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
            .map(|response| {
                let find_route_response: FindRouteResponse = response.json().unwrap();
                let mut readable_route =
                    Cursor::new(hex_utils::to_vec(&find_route_response.route).unwrap());
                Route::read(&mut readable_route).unwrap()
            })
            .map_err(|error| LightningError {
                err: error.to_string(),
                action: ErrorAction::IgnoreError,
            })
    }
}
