mod config;
mod database;
mod disk;
mod error;
mod event_handler;
mod grpc;
mod hex_utils;
mod http;
mod hybrid;
mod node;
mod services;
mod utils;

use crate::config::SenseiConfig;
use crate::database::admin::AdminDatabase;
use crate::http::admin::add_routes as add_admin_routes;
use crate::http::node::add_routes as add_node_routes;
use ::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
    Method,
};
use axum::{
    http::StatusCode,
    response::Html,
    routing::{get, get_service},
    AddExtensionLayer, Router,
};
use clap::Parser;

use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

use grpc::admin::{AdminServer, AdminService as GrpcAdminService};
use grpc::node::{NodeServer, NodeService as GrpcNodeService};
use lightning_background_processor::BackgroundProcessor;
use node::LightningNode;
use services::admin::AdminService;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tonic::transport::Server;
use tower_http::cors::{CorsLayer, Origin};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref INDEX_HTML: String = fs::read_to_string("./web-admin/build/index.html").unwrap();
}

pub struct NodeHandle {
    pub node: Arc<LightningNode>,
    pub background_processor: BackgroundProcessor,
    pub handles: Vec<JoinHandle<()>>,
}

pub type NodeDirectory = Arc<Mutex<HashMap<String, NodeHandle>>>;

#[derive(Clone)]
pub struct RequestContext {
    pub node_directory: NodeDirectory,
    pub admin_service: AdminService,
}

/// Sensei daemon
#[derive(Parser, Debug)]
#[clap(version)]
struct SenseiArgs {
    /// Sensei data directory, defaults to home directory
    #[clap(short, long)]
    data_dir: Option<String>,
    #[clap(short, long)]
    network: Option<String>,
}

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
    let config = SenseiConfig::from_file(network_config_path, Some(root_config));

    let sqlite_path = format!("{}/{}/admin.db", sensei_dir, config.network);
    let mut database = AdminDatabase::new(sqlite_path);
    database.mark_all_nodes_stopped().unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    let node_directory = Arc::new(Mutex::new(HashMap::new()));

    let admin_service = AdminService::new(
        &sensei_dir,
        config.clone(),
        node_directory.clone(),
        database,
    );

    // TODO: this seems odd too, maybe just pass around the 'admin service'
    //       and the servers will use it to get the node from the directory
    let request_context = RequestContext {
        node_directory: node_directory.clone(),
        admin_service,
    };

    let router = Router::new().route("/admin", get(live)).nest(
        "/admin/static",
        get_service(ServeDir::new("./web-admin/build/static")).handle_error(
            |error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            },
        ),
    );

    let router = add_admin_routes(router);
    let router = add_node_routes(router);

    let origins = vec![
        "http://localhost:3000".parse().unwrap(),
        "http://localhost:5401".parse().unwrap(),
    ];
    let http_service = router
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            CorsLayer::new()
                .allow_headers(vec![AUTHORIZATION, ACCEPT, COOKIE, CONTENT_TYPE])
                .allow_credentials(true)
                .allow_origin(Origin::list(origins))
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::OPTIONS,
                    Method::DELETE,
                    Method::PUT,
                    Method::PATCH,
                ]),
        )
        .layer(CookieManagerLayer::new())
        .layer(AddExtensionLayer::new(request_context.clone()))
        .into_make_service();

    let grpc_service = Server::builder()
        .add_service(NodeServer::new(GrpcNodeService {
            request_context: request_context.clone(),
        }))
        .add_service(AdminServer::new(GrpcAdminService { request_context }))
        .into_service();

    let hybrid_service = hybrid::hybrid(http_service, grpc_service);

    let server = hyper::Server::bind(&addr).serve(hybrid_service);

    println!(
        "manage your sensei node at http://localhost:{}/admin/nodes",
        config.api_port
    );

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn live() -> Html<String> {
    let index = fs::read_to_string("./web-admin/build/index.html").unwrap();
    Html(index)
}

async fn _handler() -> Html<&'static str> {
    Html(&INDEX_HTML)
}
