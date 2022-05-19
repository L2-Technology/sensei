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
use crate::error::Error as SenseiError;
use crate::lib::database::SenseiDatabase;
use crate::lib::events::SenseiEvent;
use crate::{config::SenseiConfig, hex_utils, node::LightningNode, version, NodeHandle};

use entity::access_token;
use entity::node;
use entity::sea_orm::{ActiveModelTrait, ActiveValue};
use macaroon::Macaroon;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::{collections::hash_map::Entry, fs, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;
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
}

impl AdminService {
    pub async fn new(
        data_dir: &str,
        config: SenseiConfig,
        database: SenseiDatabase,
        chain_manager: Arc<SenseiChainManager>,
        event_sender: broadcast::Sender<SenseiEvent>,
    ) -> Self {
        Self {
            data_dir: String::from(data_dir),
            config: Arc::new(config),
            node_directory: Arc::new(Mutex::new(HashMap::new())),
            database: Arc::new(database),
            chain_manager,
            event_sender,
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
                let mut port = self.config.port_range_min;
                let mut port_used_by_system = !portpicker::is_free(port);
                let mut port_used_by_sensei =
                    self.database.port_in_use(&listen_addr, port.into()).await?;

                while port <= self.config.port_range_max
                    && (port_used_by_system || port_used_by_sensei)
                {
                    port += 1;
                    port_used_by_system = !portpicker::is_free(port);
                    port_used_by_sensei =
                        self.database.port_in_use(&listen_addr, port.into()).await?;
                }

                port.into()
            }
        };

        let node_id = Uuid::new_v4().to_string();
        let (node_pubkey, node_macaroon) = LightningNode::get_node_pubkey_and_macaroon(
            node_id.clone(),
            passphrase,
            self.database.clone(),
        )
        .await?;

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

        let node = node.insert(self.database.get_connection()).await.unwrap();

        Ok((node, node_macaroon))
    }

    // note: please be sure to stop the node first? maybe?
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
}

#[cfg(test)]
mod test {
    use crate::lib::events::SenseiEvent;
    use crate::node::{HTLCStatus, LightningNode};
    use crate::services::{PaginationRequest, PaymentsFilter};
    use bitcoin::{Address, Amount, Network};
    use bitcoincore_rpc::RpcApi;
    use bitcoind::BitcoinD;
    use entity::sea_orm::{ConnectOptions, Database};
    use futures::{future, Future};
    use migration::{Migrator, MigratorTrait};
    use std::pin::Pin;
    use std::time::Instant;
    use std::{str::FromStr, sync::Arc, time::Duration};
    use tokio::runtime::{Builder, Handle, Runtime};
    use tokio::sync::broadcast;

    use crate::{
        chain::{bitcoind_client::BitcoindClient, manager::SenseiChainManager},
        config::SenseiConfig,
        lib::database::SenseiDatabase,
        services::node::{NodeRequest, NodeResponse},
    };

    use super::{AdminRequest, AdminResponse, AdminService};

    async fn fund_node(bitcoind: &BitcoinD, node: Arc<LightningNode>) {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        let fund_address = match node.call(NodeRequest::GetUnusedAddress {}).await.unwrap() {
            NodeResponse::GetUnusedAddress { address } => Some(address),
            _ => None,
        }
        .unwrap();

        let fund_address = Address::from_str(&fund_address).unwrap();

        let _res = bitcoind
            .client
            .send_to_address(
                &fund_address,
                Amount::from_btc(1.0).unwrap(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        bitcoind
            .client
            .generate_to_address(1, &miner_address)
            .unwrap();

        let closed_node = node.clone();
        let has_balance = move || {
            let wallet = closed_node.wallet.lock().unwrap();
            let balance = wallet.get_balance().unwrap();
            balance == 100_000_000
        };

        assert!(wait_until(has_balance, 15000, 250).await);
    }

    async fn create_node(
        admin_service: &AdminService,
        username: &str,
        passphrase: &str,
        start: bool,
    ) -> Arc<LightningNode> {
        let node_pubkey = match admin_service
            .call(AdminRequest::CreateNode {
                username: String::from(username),
                passphrase: String::from(passphrase),
                alias: String::from(username),
                start,
            })
            .await
            .unwrap()
        {
            AdminResponse::CreateNode {
                id,
                listen_addr,
                listen_port,
                pubkey,
                macaroon,
            } => Some(pubkey),
            _ => None,
        }
        .unwrap();

        let directory = admin_service.node_directory.lock().await;
        let handle = directory.get(&node_pubkey).unwrap();
        handle.node.clone()
    }

    async fn create_root_node(
        admin_service: &AdminService,
        username: &str,
        passphrase: &str,
        start: bool,
    ) -> Arc<LightningNode> {
        match admin_service
            .call(AdminRequest::CreateAdmin {
                username: String::from(username),
                alias: String::from(username),
                passphrase: String::from(passphrase),
                start,
            })
            .await
            .unwrap()
        {
            AdminResponse::CreateAdmin {
                pubkey,
                macaroon,
                id,
                token,
                role,
            } => {
                let directory = admin_service.node_directory.lock().await;
                let handle = directory.get(&pubkey).unwrap();
                Some(handle.node.clone())
            }
            _ => None,
        }
        .unwrap()
    }

    async fn wait_for_event<F: Fn(SenseiEvent) -> bool>(
        event_receiver: &mut broadcast::Receiver<SenseiEvent>,
        filter: F,
        timeout_ms: u64,
        interval_ms: u64,
    ) -> Option<SenseiEvent> {
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            while let Ok(event) = event_receiver.try_recv() {
                if filter(event.clone()) {
                    return Some(event);
                }
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }
        return None;
    }

    async fn wait_until<F: Fn() -> bool>(func: F, timeout_ms: u64, interval_ms: u64) -> bool {
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            if func() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }

        return false;
    }

    async fn wait_until_async<F: Future<Output = bool>, G: Fn() -> F>(
        func: G,
        timeout_ms: u64,
        interval_ms: u64,
    ) -> bool {
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            if func().await {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }

        return false;
    }

    async fn open_channel(
        bitcoind: &BitcoinD,
        from: Arc<LightningNode>,
        to: Arc<LightningNode>,
        amt_sat: u64,
    ) {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        let node_connection_string = format!(
            "{}@{}:{}",
            to.get_pubkey(),
            to.listen_addresses.first().unwrap(),
            to.listen_port
        );

        let mut event_receiver = from.event_sender.subscribe();

        from.call(NodeRequest::OpenChannel {
            node_connection_string: node_connection_string,
            amt_satoshis: amt_sat,
            public: true,
        })
        .await
        .unwrap();

        let from_node_id = from.id.clone();
        let filter = move |event| {
            if let SenseiEvent::TransactionBroadcast { node_id, .. } = event {
                if *node_id == from_node_id {
                    return true;
                }
            }
            return false;
        };

        let event = wait_for_event(&mut event_receiver, filter, 15000, 250).await;
        assert!(event.is_some());

        bitcoind
            .client
            .generate_to_address(10, &miner_address)
            .unwrap();

        let has_usable_channel = move || {
            let channels = to
                .list_channels(PaginationRequest {
                    page: 0,
                    take: 5,
                    query: None,
                })
                .unwrap()
                .0;
            channels.len() > 0 && channels[0].is_usable
        };

        assert!(wait_until(Box::new(has_usable_channel), 15000, 250).await);
    }

    async fn create_invoice(node: Arc<LightningNode>, amt_sat: u64) -> String {
        match node
            .call(NodeRequest::GetInvoice {
                amt_msat: amt_sat * 1000,
                description: String::from("test"),
            })
            .await
            .unwrap()
        {
            NodeResponse::GetInvoice { invoice } => Some(invoice),
            _ => None,
        }
        .unwrap()
    }

    async fn batch_create_invoices(
        node: Arc<LightningNode>,
        amt_sat: u64,
        num_invoices: usize,
    ) -> Vec<String> {
        let mut i = 0;
        let mut invoices: Vec<String> = vec![];
        while i < num_invoices {
            let raw_invoice = create_invoice(node.clone(), amt_sat).await;
            invoices.push(raw_invoice);
            i += 1;
        }
        invoices
    }

    async fn pay_invoice(node: Arc<LightningNode>, invoice: String) {
        node.call(NodeRequest::SendPayment { invoice })
            .await
            .unwrap();
    }

    fn setup_bitcoind() -> BitcoinD {
        let bitcoind = bitcoind::BitcoinD::new(bitcoind::downloaded_exe_path().unwrap()).unwrap();
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        bitcoind
            .client
            .generate_to_address(110, &miner_address)
            .unwrap();
        bitcoind
    }

    fn setup_test_environment(bitcoind: &BitcoinD, sensei_dir: &str) -> SenseiConfig {
        cleanup_test_environment(sensei_dir);
        std::fs::create_dir_all(format!("{}/{}", sensei_dir, Network::Regtest))
            .expect("failed to create data directory");
        let sqlite_path = format!("{}/{}/{}", sensei_dir, Network::Regtest, "sensei.sqlite");

        let mut config = SenseiConfig::default();
        config.network = Network::Regtest;
        config.bitcoind_rpc_host = bitcoind.params.rpc_socket.ip().to_string();
        config.bitcoind_rpc_port = bitcoind.params.rpc_socket.port();
        config.bitcoind_rpc_username = String::from("__cookie__");
        let cookie = std::fs::read_to_string(bitcoind.params.cookie_file.clone()).unwrap();
        let cookie_parts = cookie.split(':').collect::<Vec<&str>>();
        config.bitcoind_rpc_password = cookie_parts.last().unwrap().to_string();
        config.database_url = format!("sqlite://{}?mode=rwc", sqlite_path);
        config
    }

    fn cleanup_test_environment(sensei_dir: &str) {
        std::fs::remove_dir_all(&sensei_dir).unwrap_or_default();
    }

    async fn setup_sensei(
        sensei_dir: &str,
        bitcoind: &BitcoinD,
        persistence_handle: Handle,
    ) -> AdminService {
        let (event_sender, mut event_receiver): (
            broadcast::Sender<SenseiEvent>,
            broadcast::Receiver<SenseiEvent>,
        ) = broadcast::channel(256);
        let config = setup_test_environment(&bitcoind, sensei_dir);

        let mut db_connection_options = ConnectOptions::new(config.database_url.clone());
        db_connection_options
            .max_connections(100)
            .min_connections(10)
            .connect_timeout(Duration::new(30, 0));
        let db_connection = Database::connect(db_connection_options).await.unwrap();
        Migrator::up(&db_connection, None)
            .await
            .expect("failed to run migrations");

        let database = SenseiDatabase::new(db_connection, persistence_handle);
        database.mark_all_nodes_stopped().await.unwrap();

        let bitcoind_client = Arc::new(
            BitcoindClient::new(
                config.bitcoind_rpc_host.clone(),
                config.bitcoind_rpc_port,
                config.bitcoind_rpc_username.clone(),
                config.bitcoind_rpc_password.clone(),
                tokio::runtime::Handle::current(),
            )
            .await
            .expect("invalid bitcoind rpc config"),
        );

        let chain_manager = Arc::new(
            SenseiChainManager::new(
                config.clone(),
                bitcoind_client.clone(),
                bitcoind_client.clone(),
                bitcoind_client,
            )
            .await
            .unwrap(),
        );

        AdminService::new(
            &sensei_dir,
            config.clone(),
            database,
            chain_manager,
            event_sender,
        )
        .await
    }

    fn run_test<F>(test: fn(BitcoinD, AdminService) -> F) -> F::Output
    where
        F: Future,
    {
        let persistence_runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("persistence")
            .enable_all()
            .build()
            .unwrap();

        let persistence_runtime_handle = persistence_runtime.handle().clone();

        Builder::new_multi_thread()
            .worker_threads(10)
            .thread_name("sensei")
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let sensei_dir = String::from("./.sensei-tests");
                let bitcoind = setup_bitcoind();
                let admin_service =
                    setup_sensei(&sensei_dir, &bitcoind, persistence_runtime_handle).await;
                test(bitcoind, admin_service).await
            })
    }

    async fn smoke_test(bitcoind: BitcoinD, admin_service: AdminService) {
        let alice = create_root_node(&admin_service, "alice", "alice", true).await;
        let bob = create_node(&admin_service, "bob", "bob", true).await;
        let charlie = create_node(&admin_service, "charlie", "charlie", true).await;
        fund_node(&bitcoind, alice.clone()).await;
        fund_node(&bitcoind, bob.clone()).await;
        open_channel(&bitcoind, alice.clone(), bob.clone(), 1_000_000).await;
        open_channel(&bitcoind, bob.clone(), charlie.clone(), 1_000_000).await;

        let num_invoices = 25;

        let invoices = batch_create_invoices(charlie.clone(), 10, num_invoices).await;

        future::try_join_all(
            invoices
                .into_iter()
                .map(|invoice| pay_invoice(alice.clone(), invoice))
                .map(tokio::spawn),
        )
        .await
        .unwrap();

        let charlie_test = charlie.clone();
        let has_payments = move || {
            let pagination = PaginationRequest {
                page: 0,
                take: 1,
                query: None,
            };
            let filter = PaymentsFilter {
                status: Some(HTLCStatus::Succeeded.to_string()),
                origin: None,
            };
            let (_payments, pagination) = charlie_test
                .database
                .list_payments_sync(charlie_test.id.clone(), pagination, filter)
                .unwrap();
            pagination.total == num_invoices as u64
        };

        assert!(wait_until(has_payments, 60000, 500).await);
    }

    #[test]
    fn run_smoke_test() {
        run_test(smoke_test)
    }
}
