use std::io::Cursor;
use std::sync::Arc;

use lightning::ln::msgs::NetAddress;
use lightning::util::ser::Writeable;
use lightning::{
    routing::gossip::{NodeId, NodeInfo},
    util::ser::Readable,
};
use serde::Deserialize;
use tokio::runtime::Handle;

use crate::{error::Error, hex_utils, node::NetworkGraph};

use super::router::RemoteSenseiInfo;

pub enum NodeInfoLookup {
    Local(Arc<NetworkGraph>),
    Remote(RemoteNetworkGraph),
}

// TODO: probably want a cache in front of the remote lookup?
// these things don't change very often and the request is expensive
// perhaps even notifications from remote? like some kind of "spv network graph"
// where I can register node_ids I care about and get notified of changes
// could be whenever I make a request for one it counts as notification registration
// is this even worth it? maybe we just keep a graph in sync :eyeroll:
// or I guess a shared database/cache (redis?) could be used?
// that's probably not better then a long-lived connection with the remote node
// since they keep the graph in memory.
impl NodeInfoLookup {
    pub fn new_local(network_graph: Arc<NetworkGraph>) -> Self {
        NodeInfoLookup::Local(network_graph)
    }
    pub fn new_remote(host: String, token: String, tokio_handle: Handle) -> Self {
        NodeInfoLookup::Remote(RemoteNetworkGraph::new(host, token, tokio_handle))
    }

    pub fn get_node_info(&self, node_id: NodeId) -> Result<Option<NodeInfo>, Error> {
        match self {
            NodeInfoLookup::Local(network_graph) => {
                let network_graph = network_graph.read_only();
                Ok(network_graph.nodes().get(&node_id).cloned())
            }
            NodeInfoLookup::Remote(remote_graph) => remote_graph.get_node_info_sync(node_id),
        }
    }

    pub fn get_alias(&self, node_id: NodeId) -> Result<Option<String>, Error> {
        Ok(self
            .get_node_info(node_id)?
            .and_then(|node_info| node_info.announcement_info.map(|ann_info| ann_info.alias))
            .map(|node_alias| node_alias.to_string()))
    }

    pub fn get_addresses(&self, node_id: NodeId) -> Result<Vec<NetAddress>, Error> {
        Ok(self
            .get_node_info(node_id)?
            .and_then(|info| {
                info.announcement_info
                    .as_ref()
                    .map(|info| info.addresses.clone())
            })
            .unwrap_or_default())
    }
}

pub struct RemoteNetworkGraph {
    remote_sensei: RemoteSenseiInfo,
    tokio_handle: Handle,
}

#[derive(Deserialize)]
struct GetInfoResponse {
    node_info: Option<String>,
}

impl RemoteNetworkGraph {
    pub fn new(host: String, token: String, tokio_handle: Handle) -> Self {
        Self {
            remote_sensei: RemoteSenseiInfo { host, token },
            tokio_handle,
        }
    }

    fn get_node_info_route(&self) -> String {
        format!("{}/v1/ldk/network/node_info", self.remote_sensei.host)
    }

    fn get_node_info_sync(&self, node_id: NodeId) -> Result<Option<NodeInfo>, Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle
                .clone()
                .block_on(async move { self.get_node_info(node_id).await })
        })
    }

    async fn get_node_info(&self, node_id: NodeId) -> Result<Option<NodeInfo>, Error> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_node_info_route())
            .header("token", self.remote_sensei.token.clone())
            .json(&serde_json::json!({
              "node_id_hex": hex_utils::hex_str(&node_id.encode())
            }))
            .send()
            .await;

        match response {
            Ok(response) => {
                let get_info_response: GetInfoResponse = response.json().await.unwrap();
                Ok(get_info_response.node_info.map(|node_info| {
                    let mut readable_node_info =
                        Cursor::new(hex_utils::to_vec(&node_info).unwrap());
                    NodeInfo::read(&mut readable_node_info).unwrap()
                }))
            }
            Err(e) => Err(Error::Generic(e.to_string())),
        }
    }
}
