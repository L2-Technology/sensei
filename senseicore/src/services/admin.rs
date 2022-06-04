// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use super::{PaginationRequest, PaginationResponse};
use crate::chain::manager::SenseiChainManager;
use crate::database::SenseiDatabase;
use crate::error::Error as SenseiError;
use crate::events::SenseiEvent;
use crate::{config::SenseiConfig, hex_utils, node::LightningNode, version};

use entity::access_token;
use entity::node;
use entity::sea_orm::{ActiveModelTrait, ActiveValue};
use lightning_background_processor::BackgroundProcessor;
use macaroon::Macaroon;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::Ordering;
use std::{collections::hash_map::Entry, fs, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct NodeHandle {
    pub node: Arc<LightningNode>,
    pub background_processor: BackgroundProcessor,
    pub handles: Vec<JoinHandle<()>>,
}

pub enum AdminRequest {
    GetStatus {
        pubkey: String,
    },
    CreateAdmin {
        username: String,
        alias: String,
        passphrase: String,
        start: bool,
    },
    StartAdmin {
        passphrase: String,
    },
    CreateNode {
        username: String,
        alias: String,
        passphrase: String,
        start: bool,
    },
    ListNodes {
        pagination: PaginationRequest,
    },
    DeleteNode {
        pubkey: String,
    },
    StartNode {
        pubkey: String,
        passphrase: String,
    },
    StopNode {
        pubkey: String,
    },
    CreateToken {
        name: String,
        expires_at: u64,
        scope: String,
        single_use: bool,
    },
    ListTokens {
        pagination: PaginationRequest,
    },
    DeleteToken {
        id: String,
    },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum AdminResponse {
    GetStatus {
        version: String,
        alias: Option<String>,
        created: bool,
        running: bool,
        authenticated: bool,
        pubkey: Option<String>,
        username: Option<String>,
        role: Option<i16>,
    },
    CreateAdmin {
        pubkey: String,
        macaroon: String,
        id: String,
        role: i16,
        token: String,
    },
    StartAdmin {
        pubkey: String,
        macaroon: String,
        token: String,
    },
    CreateNode {
        pubkey: String,
        macaroon: String,
        listen_addr: String,
        listen_port: i32,
        id: String,
    },
    ListNodes {
        nodes: Vec<node::Model>,
        pagination: PaginationResponse,
    },
    DeleteNode {},
    StartNode {
        macaroon: String,
    },
    StopNode {},
    CreateToken {
        token: access_token::Model,
    },
    ListTokens {
        tokens: Vec<access_token::Model>,
        pagination: PaginationResponse,
    },
    DeleteToken {},
    Error(Error),
}

pub type NodeDirectory = Arc<Mutex<HashMap<String, NodeHandle>>>;

#[derive(Clone)]
pub struct AdminService {
    pub data_dir: String,
    pub config: Arc<SenseiConfig>,
    pub node_directory: NodeDirectory,
    pub database: Arc<SenseiDatabase>,
    pub chain_manager: Arc<SenseiChainManager>,
    pub event_sender: broadcast::Sender<SenseiEvent>,
    pub available_ports: Arc<Mutex<VecDeque<u16>>>,
}

impl AdminService {
    pub async fn new(
        data_dir: &str,
        config: SenseiConfig,
        database: SenseiDatabase,
        chain_manager: Arc<SenseiChainManager>,
        event_sender: broadcast::Sender<SenseiEvent>,
    ) -> Self {
        let mut used_ports = HashSet::new();
        let mut available_ports = VecDeque::new();
        database
            .list_ports_in_use()
            .await
            .unwrap()
            .into_iter()
            .for_each(|port| {
                used_ports.insert(port);
            });

        for port in config.port_range_min..config.port_range_max {
            if !used_ports.contains(&port) {
                available_ports.push_back(port);
            }
        }

        Self {
            data_dir: String::from(data_dir),
            config: Arc::new(config),
            node_directory: Arc::new(Mutex::new(HashMap::new())),
            database: Arc::new(database),
            chain_manager,
            event_sender,
            available_ports: Arc::new(Mutex::new(available_ports)),
        }
    }
}

#[derive(Serialize, Debug)]
pub enum Error {
    Generic(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Generic(e.to_string())
    }
}

impl From<SenseiError> for Error {
    fn from(e: SenseiError) -> Self {
        Self::Generic(e.to_string())
    }
}

impl From<macaroon::MacaroonError> for Error {
    fn from(_e: macaroon::MacaroonError) -> Self {
        Self::Generic(String::from("macaroon error"))
    }
}

impl From<migration::DbErr> for Error {
    fn from(e: migration::DbErr) -> Self {
        Self::Generic(e.to_string())
    }
}

impl AdminService {
    pub async fn call(&self, request: AdminRequest) -> Result<AdminResponse, Error> {
        match request {
            AdminRequest::GetStatus { pubkey } => {
                let root_node = self.database.get_root_node().await?;
                match root_node {
                    Some(_root_node) => {
                        let pubkey_node = self.database.get_node_by_pubkey(&pubkey).await?;
                        match pubkey_node {
                            Some(pubkey_node) => {
                                let directory = self.node_directory.lock().await;
                                let node_running = directory.contains_key(&pubkey);

                                Ok(AdminResponse::GetStatus {
                                    version: version::get_version(),
                                    alias: Some(pubkey_node.alias),
                                    created: true,
                                    running: node_running,
                                    authenticated: true,
                                    pubkey: Some(pubkey_node.pubkey),
                                    username: Some(pubkey_node.username),
                                    role: Some(pubkey_node.role),
                                })
                            }
                            None => Ok(AdminResponse::GetStatus {
                                version: version::get_version(),
                                alias: None,
                                created: true,
                                running: false,
                                authenticated: false,
                                pubkey: None,
                                username: None,
                                role: None,
                            }),
                        }
                    }
                    None => Ok(AdminResponse::GetStatus {
                        version: version::get_version(),
                        alias: None,
                        pubkey: None,
                        created: false,
                        running: false,
                        authenticated: false,
                        username: None,
                        role: None,
                    }),
                }
            }
            AdminRequest::CreateAdmin {
                username,
                alias,
                passphrase,
                start,
            } => {
                let (node, macaroon) = self
                    .create_node(username, alias, passphrase.clone(), node::NodeRole::Root)
                    .await?;

                let root_token = self.database.create_root_access_token().await.unwrap();

                let macaroon = macaroon.serialize(macaroon::Format::V2)?;

                if start {
                    self.start_node(node.clone(), passphrase).await?;
                }

                Ok(AdminResponse::CreateAdmin {
                    pubkey: node.pubkey,
                    macaroon: hex_utils::hex_str(macaroon.as_slice()),
                    id: node.id,
                    role: node.role,
                    token: root_token.token,
                })
            }
            AdminRequest::StartAdmin { passphrase } => {
                let root_node = self.database.get_root_node().await?;
                let access_token = self.database.get_root_access_token().await?;

                match root_node {
                    Some(node) => {
                        let macaroon = self.start_node(node.clone(), passphrase).await?;
                        let macaroon = macaroon.serialize(macaroon::Format::V2)?;
                        Ok(AdminResponse::StartAdmin {
                            pubkey: node.pubkey,
                            macaroon: hex_utils::hex_str(macaroon.as_slice()),
                            token: access_token.expect("no token in db").token,
                        })
                    }
                    None => Err(Error::Generic(String::from(
                        "root node not found, you need to init your sensei instance",
                    ))),
                }
            }
            AdminRequest::StartNode { pubkey, passphrase } => {
                let node = self.database.get_node_by_pubkey(&pubkey).await?;
                match node {
                    Some(node) => {
                        let macaroon = self.start_node(node, passphrase).await?;
                        let macaroon = macaroon.serialize(macaroon::Format::V2)?;
                        Ok(AdminResponse::StartNode {
                            macaroon: hex_utils::hex_str(macaroon.as_slice()),
                        })
                    }
                    None => Err(Error::Generic(String::from("node not found"))),
                }
            }
            AdminRequest::StopNode { pubkey } => {
                let node = self.database.get_node_by_pubkey(&pubkey).await?;
                match node {
                    Some(node) => {
                        self.stop_node(pubkey).await?;

                        let mut node: node::ActiveModel = node.into();
                        node.status = ActiveValue::Set(node::NodeStatus::Stopped.into());
                        node.update(self.database.get_connection()).await?;

                        Ok(AdminResponse::StopNode {})
                    }
                    None => {
                        // try stopping it anyway?
                        Ok(AdminResponse::StopNode {})
                    }
                }
            }
            AdminRequest::CreateNode {
                username,
                alias,
                passphrase,
                start,
            } => {
                let (node, macaroon) = self
                    .create_node(username, alias, passphrase.clone(), node::NodeRole::Default)
                    .await?;

                let macaroon = macaroon.serialize(macaroon::Format::V2)?;

                if start {
                    self.start_node(node.clone(), passphrase).await?;
                }
                Ok(AdminResponse::CreateNode {
                    pubkey: node.pubkey,
                    macaroon: hex_utils::hex_str(macaroon.as_slice()),
                    listen_addr: node.listen_addr,
                    listen_port: node.listen_port,
                    id: node.id,
                })
            }
            AdminRequest::ListNodes { pagination } => {
                let (nodes, pagination) = self.list_nodes(pagination).await?;
                Ok(AdminResponse::ListNodes { nodes, pagination })
            }
            AdminRequest::DeleteNode { pubkey } => {
                let node = self.database.get_node_by_pubkey(&pubkey).await?;
                match node {
                    Some(node) => {
                        self.delete_node(node).await?;
                        Ok(AdminResponse::DeleteNode {})
                    }
                    None => Err(Error::Generic(String::from("node not found"))),
                }
            }
            AdminRequest::CreateToken {
                name,
                expires_at,
                scope,
                single_use,
            } => {
                let access_token = self
                    .database
                    .create_access_token(name, scope, expires_at.try_into().unwrap(), single_use)
                    .await?;

                Ok(AdminResponse::CreateToken {
                    token: access_token,
                })
            }
            AdminRequest::ListTokens { pagination } => {
                let (tokens, pagination) = self.list_tokens(pagination).await?;
                Ok(AdminResponse::ListTokens { tokens, pagination })
            }
            AdminRequest::DeleteToken { id } => {
                self.database.delete_access_token(id).await?;
                Ok(AdminResponse::DeleteToken {})
            }
        }
    }

    async fn list_tokens(
        &self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<access_token::Model>, PaginationResponse), crate::error::Error> {
        self.database.list_access_tokens(pagination).await
    }

    async fn list_nodes(
        &self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<node::Model>, PaginationResponse), crate::error::Error> {
        self.database.list_nodes(pagination).await
    }

    async fn create_node(
        &self,
        username: String,
        alias: String,
        passphrase: String,
        role: node::NodeRole,
    ) -> Result<(node::Model, Macaroon), crate::error::Error> {
        let listen_addr = public_ip::addr().await.unwrap().to_string();

        let listen_port: i32 = match role {
            node::NodeRole::Root => 9735,
            node::NodeRole::Default => {
                let mut available_ports = self.available_ports.lock().await;
                available_ports.pop_front().unwrap().into()
            }
        };

        let node_id = Uuid::new_v4().to_string();

        let result = LightningNode::get_node_pubkey_and_macaroon(
            node_id.clone(),
            passphrase,
            self.database.clone(),
        )
        .await;

        if let Err(e) = result {
            let mut available_ports = self.available_ports.lock().await;
            available_ports.push_front(listen_port.try_into().unwrap());
            return Err(e);
        }

        let (node_pubkey, macaroon) = result.unwrap();

        let node = entity::node::ActiveModel {
            id: ActiveValue::Set(node_id),
            pubkey: ActiveValue::Set(node_pubkey),
            username: ActiveValue::Set(username),
            alias: ActiveValue::Set(alias),
            network: ActiveValue::Set(self.config.network.to_string()),
            listen_addr: ActiveValue::Set(listen_addr),
            listen_port: ActiveValue::Set(listen_port),
            role: ActiveValue::Set(role.into()),
            status: ActiveValue::Set(node::NodeStatus::Stopped.into()),
            ..Default::default()
        };

        let result = node.insert(self.database.get_connection()).await;

        if let Err(e) = result {
            let mut available_ports = self.available_ports.lock().await;
            available_ports.push_front(listen_port.try_into().unwrap());
            return Err(e.into());
        }

        let node = result.unwrap();

        Ok((node, macaroon))
    }

    // note: please be sure to stop the node first? maybe?
    // TODO: this was never updated with the DB rewrite
    //       need to release the port and actually delete the node
    async fn delete_node(&self, node: node::Model) -> Result<(), crate::error::Error> {
        let data_dir = format!("{}/{}/{}", self.data_dir, self.config.network, node.id);
        Ok(fs::remove_dir_all(&data_dir)?)
    }

    async fn start_node(
        &self,
        node: node::Model,
        passphrase: String,
    ) -> Result<Macaroon, crate::error::Error> {
        let mut node_directory = self.node_directory.lock().await;

        let (network_graph, network_graph_msg_handler, external_router) = match node.get_role() {
            node::NodeRole::Root => (None, None, false),
            node::NodeRole::Default => {
                if let Some(root_node) = self.database.get_root_node().await? {
                    let root_pubkey = root_node.pubkey;
                    if let Entry::Occupied(entry) = node_directory.entry(root_pubkey) {
                        let root_node_handle = entry.get();
                        let network_graph = root_node_handle.node.network_graph.clone();
                        let network_graph_message_handler =
                            root_node_handle.node.network_graph_msg_handler.clone();
                        (
                            Some(network_graph),
                            Some(network_graph_message_handler),
                            true,
                        )
                    } else {
                        return Err(crate::error::Error::AdminNodeNotStarted);
                    }
                } else {
                    return Err(crate::error::Error::AdminNodeNotCreated);
                }
            }
        };

        match node_directory.entry(node.pubkey.clone()) {
            Entry::Vacant(entry) => {
                let lightning_node = LightningNode::new(
                    self.config.clone(),
                    node.id.clone(),
                    vec![node.listen_addr.clone()],
                    node.listen_port.try_into().unwrap(),
                    node.alias.clone(),
                    format!(
                        "{}/{}/{}",
                        self.data_dir,
                        self.config.network,
                        node.id.clone()
                    ),
                    passphrase,
                    external_router,
                    network_graph,
                    network_graph_msg_handler,
                    self.chain_manager.clone(),
                    self.database.clone(),
                    self.event_sender.clone(),
                )
                .await?;

                println!(
                    "starting node {} on port {}",
                    node.pubkey.clone(),
                    node.listen_port
                );

                let (handles, background_processor) = lightning_node.clone().start().await;

                entry.insert(NodeHandle {
                    node: Arc::new(lightning_node.clone()),
                    background_processor,
                    handles,
                });

                let mut node: node::ActiveModel = node.into();
                node.status = ActiveValue::Set(node::NodeStatus::Running.into());
                node.listen_addr = ActiveValue::Set(public_ip::addr().await.unwrap().to_string());
                node.save(self.database.get_connection()).await?;
                Ok(lightning_node.macaroon)
            }
            Entry::Occupied(entry) => {
                // TODO: verify passphrase
                Ok(entry.get().node.macaroon.clone())
            }
        }
    }

    async fn stop_node(&self, pubkey: String) -> Result<(), crate::error::Error> {
        let mut node_directory = self.node_directory.lock().await;
        let entry = node_directory.entry(pubkey.clone());

        if let Entry::Occupied(entry) = entry {
            let node_handle = entry.remove();

            // Disconnect our peers and stop accepting new connections. This ensures we don't continue
            // updating our channel data after we've stopped the background processor.
            node_handle.node.peer_manager.disconnect_all_peers();
            node_handle.node.stop_listen.store(true, Ordering::Release);
            let _res = node_handle.background_processor.stop();
            for handle in node_handle.handles {
                handle.abort();
            }
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), crate::error::Error> {
        let pubkeys = {
            let node_directory = self.node_directory.lock().await;
            node_directory.keys().cloned().collect::<Vec<String>>()
        };

        for pubkey in pubkeys.into_iter() {
            self.stop_node(pubkey).await.unwrap();
        }

        self.chain_manager.stop().await;

        Ok(())
    }
}
