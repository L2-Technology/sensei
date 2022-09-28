// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

mod grpc;
mod http;
mod hybrid;

use senseicore::{
    chain::{
        bitcoind_client::BitcoindClient, manager::SenseiChainManager, AnyBlockSource,
        AnyBroadcaster, AnyFeeEstimator,
    },
    config::SenseiConfig,
    database::SenseiDatabase,
    events::{AnyNotifier, EventService, SenseiEvent},
    services::admin::{AdminRequest, AdminResponse, AdminService},
    version,
};

use entity::sea_orm::{self, ConnectOptions};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

use crate::http::admin::add_routes as add_admin_routes;
use crate::http::node::add_routes as add_node_routes;

use ::http::{header, Uri};
use axum::{
    body::{boxed, Full},
    extract::Extension,
    handler::Handler,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
use clap::Parser;

use rust_embed::RustEmbed;

use grpc::admin::{AdminServer, AdminService as GrpcAdminService};
use grpc::node::{NodeServer, NodeService as GrpcNodeService};
use std::time::Duration;
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
};
use tower_cookies::CookieManagerLayer;

use std::fs;
use std::sync::Arc;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;

use tokio::runtime::Builder;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

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
    #[clap(long, env = "PORT_RANGE_MIN")]
    port_range_min: Option<u16>,
    #[clap(long, env = "PORT_RANGE_MAX")]
    port_range_max: Option<u16>,
    #[clap(long, env = "API_HOST")]
    api_host: Option<String>,
    #[clap(long, env = "API_PORT")]
    api_port: Option<u16>,
    #[clap(long, env = "DATABASE_URL")]
    database_url: Option<String>,
    #[clap(long, env = "REMOTE_P2P_HOST")]
    remote_p2p_host: Option<String>,
    #[clap(long, env = "REMOTE_P2P_TOKEN")]
    remote_p2p_token: Option<String>,
    #[clap(long, env = "INSTANCE_NAME")]
    instance_name: Option<String>,
    #[clap(long, env = "REMOTE_CHAIN_HOST")]
    remote_chain_host: Option<String>,
    #[clap(long, env = "REMOTE_CHAIN_TOKEN")]
    remote_chain_token: Option<String>,
    #[clap(long, env = "HTTP_NOTIFIER_URL")]
    http_notifier_url: Option<String>,
    #[clap(long, env = "HTTP_NOTIFIER_TOKEN")]
    http_notifier_token: Option<String>,
    #[clap(long, env = "REGION")]
    region: Option<String>,
    #[clap(long, env = "POLL_FOR_CHAIN_UPDATES")]
    poll_for_chain_updates: Option<bool>,
    #[clap(long, env = "ALLOW_ORIGINS")]
    allow_origins: Option<Vec<String>>,
    #[clap(long, env = "RAPID_GOSSIP_SYNC_SERVER_HOST")]
    rapid_gossip_sync_server_host: Option<String>,
}

pub type AdminRequestResponse = (AdminRequest, Sender<AdminResponse>);
fn main() {
    env_logger::init();
    macaroon::initialize().expect("failed to initialize macaroons");
    let args = SenseiArgs::parse();

    let stop_signal = Arc::new(AtomicBool::new(false));

    for term_signal in signal_hook::consts::TERM_SIGNALS {
        signal_hook::flag::register(*term_signal, Arc::clone(&stop_signal)).unwrap();
    }

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
    if let Some(api_host) = args.api_host {
        config.api_host = api_host;
    }
    if let Some(database_url) = args.database_url {
        config.database_url = database_url;
    }
    if let Some(remote_p2p_host) = args.remote_p2p_host {
        config.remote_p2p_host = Some(remote_p2p_host);
    }
    if let Some(remote_p2p_token) = args.remote_p2p_token {
        config.remote_p2p_token = Some(remote_p2p_token);
    }
    if let Some(instance_name) = args.instance_name {
        config.instance_name = instance_name;
    }
    if let Some(remote_chain_host) = args.remote_chain_host {
        config.remote_chain_host = Some(remote_chain_host);
    }
    if let Some(remote_chain_token) = args.remote_chain_token {
        config.remote_chain_token = Some(remote_chain_token);
    }
    if let Some(http_notifier_url) = args.http_notifier_url {
        config.http_notifier_url = Some(http_notifier_url);
    }
    if let Some(http_notifier_token) = args.http_notifier_token {
        config.http_notifier_token = Some(http_notifier_token);
    }
    if let Some(region) = args.region {
        config.region = Some(region)
    }
    if let Some(poll_for_chain_updates) = args.poll_for_chain_updates {
        config.poll_for_chain_updates = poll_for_chain_updates
    }
    if let Some(rapid_gossip_sync_server_host) = args.rapid_gossip_sync_server_host {
        config.rapid_gossip_sync_server_host = Some(rapid_gossip_sync_server_host)
    }

    if !config.database_url.starts_with("postgres:") && !config.database_url.starts_with("mysql:") {
        let sqlite_path = format!("{}/{}/{}", sensei_dir, config.network, config.database_url);
        config.database_url = format!("sqlite://{}?mode=rwc", sqlite_path);
    }

    let persistence_runtime = Builder::new_multi_thread()
        .worker_threads(20)
        .thread_name("persistence")
        .enable_all()
        .build()
        .unwrap();
    let persistence_runtime_handle = persistence_runtime.handle().clone();

    let sensei_runtime = Builder::new_multi_thread()
        .worker_threads(20)
        .thread_name("sensei")
        .enable_all()
        .build()
        .unwrap();

    sensei_runtime.block_on(async move {
        let (event_sender, event_receiver): (
            broadcast::Sender<SenseiEvent>,
            broadcast::Receiver<SenseiEvent>,
        ) = broadcast::channel(1024);

        let mut db_connection_options = ConnectOptions::new(config.database_url.clone());
        db_connection_options
            .max_connections(100)
            .min_connections(10)
            .connect_timeout(Duration::new(30, 0));
        let db_connection = Database::connect(db_connection_options).await.unwrap();
        Migrator::up(&db_connection, None)
            .await
            .expect("failed to run migrations");

        let database = SenseiDatabase::new(db_connection, persistence_runtime_handle);

        let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));

        let notifier = match (
            config.http_notifier_url.as_ref(),
            config.http_notifier_token.as_ref(),
        ) {
            (Some(url), Some(token)) => AnyNotifier::new_http(url.clone(), token.clone()),
            _ => AnyNotifier::new_log(),
        };

        EventService::listen(tokio::runtime::Handle::current(), notifier, event_receiver);

        let (block_source, fee_estimator, broadcaster) = match (
            config.remote_chain_host.as_ref(),
            config.remote_chain_token.as_ref(),
        ) {
            (Some(host), Some(token)) => (
                AnyBlockSource::new_remote(config.network, host.clone(), token.clone()),
                AnyFeeEstimator::new_remote(
                    host.clone(),
                    token.clone(),
                    tokio::runtime::Handle::current(),
                ),
                AnyBroadcaster::new_remote(
                    host.clone(),
                    token.clone(),
                    tokio::runtime::Handle::current(),
                ),
            ),
            _ => {
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

                (
                    AnyBlockSource::Local(bitcoind_client.clone()),
                    AnyFeeEstimator::Local(bitcoind_client.clone()),
                    AnyBroadcaster::Local(bitcoind_client),
                )
            }
        };

        let chain_manager = Arc::new(
            SenseiChainManager::new(
                config.clone(),
                Arc::new(block_source),
                Arc::new(fee_estimator),
                Arc::new(broadcaster),
            )
            .await
            .unwrap(),
        );

        let admin_service_stop_signal = stop_signal.clone();
        let admin_service = Arc::new(
            AdminService::new(
                &sensei_dir,
                config.clone(),
                database,
                chain_manager,
                event_sender.clone(),
                tokio::runtime::Handle::current(),
                admin_service_stop_signal,
            )
            .await,
        );

        let api_router = Router::new();
        let api_router = add_admin_routes(api_router);
        let api_router = add_node_routes(api_router);

        let router = Router::new()
            .nest("/api", api_router)
            .fallback(static_handler.into_service());

        let cors_layer = CorsLayer::very_permissive().allow_credentials(true);

        let mut allow_origins: Vec<String> = vec![];
        if let Some(mut origins) = args.allow_origins {
            allow_origins.append(&mut origins);
        }
        let cors_layer = cors_layer.allow_origin(
            allow_origins
                .into_iter()
                .map(|o| o.parse().unwrap())
                .collect::<Vec<HeaderValue>>(),
        );

        let router = router.layer(cors_layer);

        let port = format!("{}", config.api_port);

        let http_service = router
            .layer(CookieManagerLayer::new())
            .layer(Extension(admin_service.clone()))
            .into_make_service();

        let grpc_service = Server::builder()
            .add_service(NodeServer::new(GrpcNodeService {
                admin_service: admin_service.clone(),
            }))
            .add_service(AdminServer::new(GrpcAdminService {
                admin_service: admin_service.clone(),
            }))
            .into_service();

        let hybrid_service = hybrid::hybrid(http_service, grpc_service);

        let server = hyper::Server::bind(&addr).serve(hybrid_service);

        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("server errored with: {:?}", e);
            }
        });

        println!(
            "manage your sensei node at http://{}:{}/admin/nodes",
            config.api_host.clone(),
            port.clone()
        );

        let _res = event_sender.send(SenseiEvent::InstanceStarted {
            instance_name: config.instance_name.clone(),
            api_host: config.api_host.clone(),
            api_port: config.api_port,
            network: config.network.to_string(),
            version: version::get_version(),
            region: config.region,
        });

        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            if stop_signal.load(Ordering::Acquire) {
                let _res = admin_service.stop().await;
                break;
            }
        }

        let _res = event_sender.send(SenseiEvent::InstanceStopped {
            instance_name: config.instance_name.clone(),
            api_host: config.api_host.clone(),
        });
    });
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    let paths_to_passthrough = ["static/", "images/"];
    let files_to_passthrough = [
        "favicon.ico",
        "favicon-16x16.png",
        "favicon-32x32.png",
        "logo192.png",
        "logo512.png",
        "manifest.json",
    ];
    let mut passthrough = false;

    paths_to_passthrough.iter().for_each(|pp| {
        if path.starts_with(pp) {
            passthrough = true;
        }
    });

    if files_to_passthrough.contains(&path.as_str()) {
        passthrough = true;
    }

    if path.starts_with("admin/static/") {
        path = path.replace("admin/static/", "static/");
        passthrough = true;
    }

    if !passthrough {
        path = String::from("index.html");
    }

    StaticFile(path)
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
