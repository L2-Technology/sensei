// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

mod chain;
mod config;
mod database;
mod disk;
mod error;
mod event_handler;
mod grpc;
mod hex_utils;
mod http;
mod hybrid;
mod lib;
mod node;
mod services;
mod utils;

use crate::chain::bitcoind_client::BitcoindClient;
use crate::{config::SenseiConfig, chain::manager::SenseiChainManager};
use crate::database::admin::AdminDatabase;
use crate::http::admin::add_routes as add_admin_routes;
use crate::http::node::add_routes as add_node_routes;
use ::http::{
    header::{self, ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
    Method, Uri,
};
use axum::{
    body::{boxed, Full},
    handler::Handler,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    AddExtensionLayer, Router,
};
use clap::Parser;
use rust_embed::RustEmbed;

use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;

use grpc::admin::{AdminServer, AdminService as GrpcAdminService};
use grpc::node::{NodeServer, NodeService as GrpcNodeService};
use lightning_background_processor::BackgroundProcessor;
use node::LightningNode;
use services::admin::{AdminRequest, AdminResponse, AdminService};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tonic::transport::Server;
use tower_http::cors::{CorsLayer, Origin};

use tokio::sync::mpsc::Sender;

pub struct NodeHandle {
    pub node: Arc<LightningNode>,
    pub background_processor: BackgroundProcessor,
    pub handles: Vec<JoinHandle<()>>,
}

pub type NodeDirectory = Arc<Mutex<HashMap<String, NodeHandle>>>;

pub struct RequestContext {
    pub node_directory: NodeDirectory,
    pub admin_service: AdminService,
}

/// Sensei daemon
#[derive(Parser, Debug)]
#[clap(version)]
struct SenseiArgs {
    /// Sensei data directory, defaults to home directory
    #[clap(long, env = "DATA_DIR")]
    data_dir: Option<String>,
    #[clap(long, env = "NETWORK")]
    network: Option<String>,
    #[clap(long, env = "BITCOIND_RPC_HOST")]
    bitcoind_rpc_host: Option<String>,
    #[clap(long, env = "BITCOIND_RPC_PORT")]
    bitcoind_rpc_port: Option<u16>,
    #[clap(long, env = "BITCOIND_RPC_USERNAME")]
    bitcoind_rpc_username: Option<String>,
    #[clap(long, env = "BITCOIND_RPC_PASSWORD")]
    bitcoind_rpc_password: Option<String>,
    #[clap(long, env = "DEVELOPMENT_MODE")]
    development_mode: Option<bool>,
    #[clap(long, env = "PORT_RANGE_MIN")]
    port_range_min: Option<u16>,
    #[clap(long, env = "PORT_RANGE_MAX")]
    port_range_max: Option<u16>,
    #[clap(long, env = "API_PORT")]
    api_port: Option<u16>,
}

pub type AdminRequestResponse = (AdminRequest, Sender<AdminResponse>);

#[tokio::main]
async fn main() {
    macaroon::initialize().expect("failed to initialize macaroons");
    let args = SenseiArgs::parse();

    let sensei_dir = match args.data_dir {
        Some(dir) => dir,
        None => {
            let home_dir = dirs::home_dir().unwrap_or_else(|| ".".into());
            format!("{}/.sensei", home_dir.to_str().unwrap())
        }
    };

    fs::create_dir_all(&sensei_dir).expect("failed to create data directory");

    let root_config_path = format!("{}/config.json", sensei_dir);
    let mut root_config = SenseiConfig::from_file(root_config_path, None);

    if let Some(network) = args.network {
        root_config.set_network(network.parse::<bitcoin::Network>().unwrap());
    }

    fs::create_dir_all(format!("{}/{}", sensei_dir, root_config.network))
        .expect("failed to create network directory");

    let network_config_path = format!("{}/{}/config.json", sensei_dir, root_config.network);
    let mut config = SenseiConfig::from_file(network_config_path, Some(root_config));

    // override config with command line arguments or ENV vars
    if let Some(bitcoind_rpc_host) = args.bitcoind_rpc_host {
        config.bitcoind_rpc_host = bitcoind_rpc_host
    }
    if let Some(bitcoind_rpc_port) = args.bitcoind_rpc_port {
        config.bitcoind_rpc_port = bitcoind_rpc_port
    }
    if let Some(bitcoind_rpc_username) = args.bitcoind_rpc_username {
        config.bitcoind_rpc_username = bitcoind_rpc_username
    }
    if let Some(bitcoind_rpc_password) = args.bitcoind_rpc_password {
        config.bitcoind_rpc_password = bitcoind_rpc_password
    }
    if let Some(port_range_min) = args.port_range_min {
        config.port_range_min = port_range_min;
    }
    if let Some(port_range_max) = args.port_range_max {
        config.port_range_max = port_range_max;
    }
    if let Some(api_port) = args.api_port {
        config.api_port = api_port;
    }

    let sqlite_path = format!("{}/{}/admin.db", sensei_dir, config.network);
    let mut database = AdminDatabase::new(sqlite_path);
    database.mark_all_nodes_stopped().unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    let node_directory = Arc::new(Mutex::new(HashMap::new()));

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

    let chain_manager = Arc::new(SenseiChainManager::new(
        config.clone(),
        bitcoind_client.clone(),
        bitcoind_client.clone(),
        bitcoind_client
    ).await.unwrap());

    let admin_service = AdminService::new(
        &sensei_dir,
        config.clone(),
        node_directory.clone(),
        database,
        chain_manager
    )
    .await;

    // TODO: this seems odd too, maybe just pass around the 'admin service'
    //       and the servers will use it to get the node from the directory
    let request_context = Arc::new(RequestContext {
        node_directory: node_directory.clone(),
        admin_service,
    });

    let router = Router::new()
        .route("/admin/*path", static_handler.into_service())
        .fallback(get(not_found));

    let router = add_admin_routes(router);
    let router = add_node_routes(router);

    let router = match args.development_mode {
        Some(_development_mode) => router.layer(
            CorsLayer::new()
                .allow_headers(vec![AUTHORIZATION, ACCEPT, COOKIE, CONTENT_TYPE])
                .allow_credentials(true)
                .allow_origin(Origin::list(vec![
                    "http://localhost:3001".parse().unwrap(),
                    "http://localhost:5401".parse().unwrap(),
                ]))
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::OPTIONS,
                    Method::DELETE,
                    Method::PUT,
                    Method::PATCH,
                ]),
        ),
        None => router,
    };

    let port = match args.development_mode {
        Some(_) => String::from("3001"),
        None => format!("{}", config.api_port),
    };

    let http_service = router
        .layer(CookieManagerLayer::new())
        .layer(AddExtensionLayer::new(request_context.clone()))
        .into_make_service();

    let grpc_service = Server::builder()
        .add_service(NodeServer::new(GrpcNodeService {
            request_context: request_context.clone(),
        }))
        .add_service(AdminServer::new(GrpcAdminService {
            request_context: request_context.clone(),
        }))
        .into_service();

    let hybrid_service = hybrid::hybrid(http_service, grpc_service);

    let server = hyper::Server::bind(&addr).serve(hybrid_service);

    println!(
        "manage your sensei node at http://localhost:{}/admin/nodes",
        port
    );

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

// We use a wildcard matcher ("/static/*file") to match against everything
// within our defined assets directory. This is the directory on our Asset
// struct below, where folder = "examples/public/".
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("admin/static/") {
        path = path.replace("admin/static/", "static/");
    } else {
        path = String::from("index.html");
    }

    StaticFile(path)
}

// Finally, we use a fallback route for anything that didn't match.
async fn not_found() -> Html<&'static str> {
    Html("<h1>404</h1><p>Not Found</p>")
}

#[derive(RustEmbed)]
#[folder = "web-admin/build/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}
