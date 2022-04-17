// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::chain::broadcaster::SenseiBroadcaster;
use crate::chain::fee_estimator::SenseiFeeEstimator;
use crate::chain::listener_database::ListenerDatabase;
use crate::chain::manager::SenseiChainManager;
use crate::config::LightningNodeConfig;
use crate::database::node::NodeDatabase;
use crate::disk::{DataPersister, FilesystemLogger};
use crate::error::Error;
use crate::event_handler::LightningNodeEventHandler;
use crate::lib::network_graph::OptionalNetworkGraphMsgHandler;
use crate::services::node::{Channel, NodeInfo, NodeRequest, NodeRequestError, NodeResponse, Peer};
use crate::services::{PaginationRequest, PaginationResponse, PaymentsFilter};
use crate::utils::PagedVec;
use crate::{database, disk, hex_utils};
use bdk::database::SqliteDatabase;
use bdk::keys::ExtendedKey;
use bdk::wallet::AddressIndex;
use bdk::TransactionDetails;
use bitcoin::hashes::Hash;
use lightning::chain::channelmonitor::ChannelMonitor;

use lightning::ln::msgs::NetAddress;
use lightning_invoice::payment::PaymentError;
use tindercrypt::cryptors::RingCryptor;

use bdk::template::DescriptorTemplateOut;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::PublicKey;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use bitcoin::BlockHash;
use lightning::chain::chainmonitor;
use lightning::chain::keysinterface::{InMemorySigner, KeysInterface, KeysManager, Recipient};
use lightning::chain::Watch;
use lightning::chain::{self, Filter};
use lightning::ln::channelmanager::{self, ChannelDetails, SimpleArcChannelManager};
use lightning::ln::channelmanager::{ChainParameters, ChannelManagerReadArgs};
use lightning::ln::peer_handler::{
    IgnoringMessageHandler, MessageHandler, PeerManager as LdkPeerManager,
};
use lightning::ln::{PaymentHash, PaymentPreimage, PaymentSecret};
use lightning::routing::network_graph::{NetGraphMsgHandler, NetworkGraph, NodeId};
use lightning::routing::scoring::{ProbabilisticScorer, ProbabilisticScorerUsingTime};
use lightning::util::config::{ChannelConfig, ChannelHandshakeLimits, UserConfig};
use lightning::util::ser::ReadableArgs;
use lightning_background_processor::BackgroundProcessor;
use lightning_invoice::utils::DefaultRouter;
use lightning_invoice::{payment, utils, Currency, Invoice};
use lightning_net_tokio::SocketDescriptor;
use lightning_persister::FilesystemPersister;
use macaroon::Macaroon;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use std::{fmt, fs};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum HTLCStatus {
    Pending,
    Succeeded,
    Failed,
    Unknown,
}

impl Display for HTLCStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            HTLCStatus::Pending => "pending".to_string(),
            HTLCStatus::Succeeded => "succeeded".to_string(),
            HTLCStatus::Failed => "failed".to_string(),
            HTLCStatus::Unknown => "unknown".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PaymentOrigin {
    InvoiceIncoming,
    InvoiceOutgoing,
    SpontaneousIncoming,
    SpontaneousOutgoing,
}

impl Display for PaymentOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            PaymentOrigin::InvoiceIncoming => "invoice_incoming".to_string(),
            PaymentOrigin::InvoiceOutgoing => "invoice_outgoing".to_string(),
            PaymentOrigin::SpontaneousIncoming => "spontaneous_incoming".to_string(),
            PaymentOrigin::SpontaneousOutgoing => "spontaneous_outgoing".to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Serialize)]
pub struct MillisatAmount(pub Option<u64>);

impl fmt::Display for MillisatAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(amt) => write!(f, "{}", amt),
            None => write!(f, "unknown"),
        }
    }
}

#[derive(Clone)]
pub struct PaymentInfo {
    pub hash: PaymentHash,
    pub preimage: Option<PaymentPreimage>,
    pub secret: Option<PaymentSecret>,
    pub status: HTLCStatus,
    pub amt_msat: MillisatAmount,
    pub origin: PaymentOrigin,
    pub label: Option<String>,
    pub invoice: Option<String>,
}

pub type ChainMonitor = chainmonitor::ChainMonitor<
    InMemorySigner,
    Arc<dyn Filter + Send + Sync>,
    Arc<SenseiBroadcaster>,
    Arc<SenseiFeeEstimator>,
    Arc<FilesystemLogger>,
    Arc<FilesystemPersister>,
>;

trait MustSized: Sized {}

pub type SimpleArcPeerManager<SD, M, T, F, L> = LdkPeerManager<
    SD,
    Arc<SimpleArcChannelManager<M, T, F, L>>,
    Arc<OptionalNetworkGraphMsgHandler>,
    Arc<L>,
    Arc<IgnoringMessageHandler>,
>;

pub type PeerManager = SimpleArcPeerManager<
    SocketDescriptor,
    ChainMonitor,
    SenseiBroadcaster,
    SenseiFeeEstimator,
    FilesystemLogger,
>;

pub type ChannelManager =
    SimpleArcChannelManager<ChainMonitor, SenseiBroadcaster, SenseiFeeEstimator, FilesystemLogger>;

pub type Router = DefaultRouter<Arc<NetworkGraph>, Arc<FilesystemLogger>>;

pub type InvoicePayer = payment::InvoicePayer<
    Arc<ChannelManager>,
    Router,
    Arc<Mutex<ProbabilisticScorer<Arc<NetworkGraph>>>>,
    Arc<FilesystemLogger>,
    Arc<LightningNodeEventHandler>,
>;

#[allow(dead_code)]
pub type SyncableMonitor = (
    ChannelMonitor<InMemorySigner>,
    Arc<SenseiBroadcaster>,
    Arc<SenseiFeeEstimator>,
    Arc<FilesystemLogger>,
);

pub type NetworkGraphMessageHandler = NetGraphMsgHandler<
    Arc<NetworkGraph>,
    Arc<dyn chain::Access + Send + Sync>,
    Arc<FilesystemLogger>,
>;

fn get_wpkh_descriptors_for_extended_key(
    xkey: ExtendedKey,
    network: Network,
    base_path: &str,
    account_number: u32,
) -> (DescriptorTemplateOut, DescriptorTemplateOut) {
    let master_xprv = xkey.into_xprv(network).unwrap();
    let coin_type = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };

    let base_path = DerivationPath::from_str(base_path).unwrap();
    let derivation_path = base_path.extend(&[
        ChildNumber::from_hardened_idx(coin_type).unwrap(),
        ChildNumber::from_hardened_idx(account_number).unwrap(),
    ]);

    let receive_descriptor_template = bdk::descriptor!(wpkh((
        master_xprv,
        derivation_path.extend(&[ChildNumber::Normal { index: 0 }])
    )))
    .unwrap();
    let change_descriptor_template = bdk::descriptor!(wpkh((
        master_xprv,
        derivation_path.extend(&[ChildNumber::Normal { index: 1 }])
    )))
    .unwrap();

    (receive_descriptor_template, change_descriptor_template)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MacaroonSession {
    pub identifier: Vec<u8>,
    pub pubkey: String,
}

impl MacaroonSession {
    pub fn new(macaroon: &Macaroon) -> Result<MacaroonSession, Error> {
        let identifier_bytes = macaroon.identifier().0;
        let session_string = String::from_utf8_lossy(&identifier_bytes);
        serde_json::from_str(&session_string).map_err(|_e| Error::InvalidMacaroon)
    }
}

#[derive(Clone)]
pub struct LightningNode {
    pub config: LightningNodeConfig,
    pub macaroon: Macaroon,
    pub seed: [u8; 32],
    pub database: Arc<Mutex<NodeDatabase>>,
    pub wallet: Arc<Mutex<bdk::Wallet<(), SqliteDatabase>>>,
    pub channel_manager: Arc<ChannelManager>,
    pub chain_monitor: Arc<ChainMonitor>,
    pub chain_manager: Arc<SenseiChainManager>,
    pub peer_manager: Arc<PeerManager>,
    pub network_graph: Arc<NetworkGraph>,
    pub network_graph_msg_handler: Arc<NetworkGraphMessageHandler>,
    pub keys_manager: Arc<KeysManager>,
    pub logger: Arc<FilesystemLogger>,
    pub invoice_payer: Arc<InvoicePayer>,
    pub scorer: Arc<Mutex<ProbabilisticScorerUsingTime<Arc<NetworkGraph>, Instant>>>,
    pub stop_listen: Arc<AtomicBool>,
}

impl LightningNode {
    fn find_or_create_seed(
        passphrase: String,
        database: &mut NodeDatabase,
    ) -> Result<[u8; 32], Error> {
        let cryptor = RingCryptor::new();

        let mut seed: [u8; 32] = [0; 32];
        match database.get_seed()? {
            Some(encrypted_seed) => {
                let decrypted_seed =
                    cryptor.open(passphrase.as_bytes(), encrypted_seed.as_slice())?;

                if decrypted_seed.len() != 32 {
                    return Err(Error::InvalidSeedLength);
                }
                seed.copy_from_slice(decrypted_seed.as_slice());
            }
            None => {
                thread_rng().fill_bytes(&mut seed);
                let encrypted_seed = cryptor.seal_with_passphrase(passphrase.as_bytes(), &seed)?;
                database.create_seed(encrypted_seed)?;
            }
        }
        Ok(seed)
    }

    fn find_or_create_macaroon(
        seed: &[u8],
        pubkey: String,
        macaroon_path: String,
        database: &mut NodeDatabase,
    ) -> Result<Macaroon, Error> {
        match File::open(macaroon_path.clone()) {
            Ok(mut file) => {
                let mut bytes: Vec<u8> = Vec::new();
                let _bytes_read = file.read_to_end(&mut bytes)?;
                let macaroon = macaroon::Macaroon::deserialize(bytes.as_slice())?;
                Ok(macaroon)
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    let macaroon_identifier = uuid::Uuid::new_v4().as_bytes().to_vec();
                    database.create_macaroon(macaroon_identifier.clone())?;
                    let macaroon_data = MacaroonSession {
                        pubkey,
                        identifier: macaroon_identifier,
                    };
                    let macaroon_data = serde_json::to_string(&macaroon_data).unwrap();

                    let macaroon_key = macaroon::MacaroonKey::from(seed);
                    let macaroon_identifier = macaroon::ByteString::from(macaroon_data);

                    let admin_macaroon = macaroon::Macaroon::create(
                        Some("senseid".to_string()),
                        &macaroon_key,
                        macaroon_identifier,
                    )?;
                    let serialized_macaroon = admin_macaroon.serialize(macaroon::Format::V2)?;
                    match File::create(macaroon_path.clone()) {
                        Ok(mut f) => {
                            f.write_all(serialized_macaroon.as_slice())?;
                            f.sync_all()?;
                            Ok(admin_macaroon)
                        }
                        Err(e) => {
                            println!(
                                "ERROR: Unable to create admin.macaroon file {}: {}",
                                macaroon_path, e
                            );
                            Err(Error::FailedToWriteSeed)
                        }
                    }
                }
                _ => Err(Error::Io(e)),
            },
        }
    }

    pub async fn verify_macaroon(
        &self,
        macaroon: Macaroon,
        session: MacaroonSession,
    ) -> Result<(), Error> {
        let mut database = self.database.lock().unwrap();
        let exists = database
            .macaroon_exists(session.identifier)
            .map_err(Error::Db)?;
        if !exists {
            return Err(Error::InvalidMacaroon);
        }

        let verifier = macaroon::Verifier::default();
        let key = macaroon::MacaroonKey::from(&self.seed[..]);
        verifier
            .verify(&macaroon, &key, vec![])
            .map_err(|_e| Error::InvalidMacaroon)
    }

    pub async fn new(
        config: LightningNodeConfig,
        network_graph: Option<Arc<NetworkGraph>>,
        network_graph_msg_handler: Option<Arc<NetworkGraphMessageHandler>>,
        chain_manager: Arc<SenseiChainManager>,
    ) -> Result<Self, Error> {
        let data_dir = config.data_dir();
        fs::create_dir_all(data_dir.clone())?;
        let mut node_database = NodeDatabase::new(config.node_database_path());

        let network = config.network;
        let channel_manager_path = config.channel_manager_path();
        let admin_macaroon_path = config.admin_macaroon_path();

        let seed =
            LightningNode::find_or_create_seed(config.passphrase.clone(), &mut node_database)?;

        let cur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let xprivkey = ExtendedPrivKey::new_master(network, &seed).unwrap();
        let xkey = ExtendedKey::from(xprivkey);
        let native_segwit_base_path = "m/84";
        let account_number = 0;
        let (receive_descriptor_template, change_descriptor_template) =
            get_wpkh_descriptors_for_extended_key(
                xkey,
                network,
                native_segwit_base_path,
                account_number,
            );

        let database = SqliteDatabase::new(config.bdk_database_path());

        let bdk_wallet = Arc::new(Mutex::new(bdk::Wallet::new_offline(
            receive_descriptor_template,
            Some(change_descriptor_template),
            network,
            database,
        )?));

        let listener_database =
            ListenerDatabase::new(config.bdk_database_path(), config.node_database_path());

        let logger = Arc::new(FilesystemLogger::new(data_dir.clone()));

        let fee_estimator = Arc::new(SenseiFeeEstimator {
            fee_estimator: chain_manager.bitcoind_client.clone(),
        });

        let broadcaster = Arc::new(SenseiBroadcaster {
            broadcaster: chain_manager.bitcoind_client.clone(),
            listener_database: listener_database.clone(),
        });

        let persister = Arc::new(FilesystemPersister::new(data_dir));

        let keys_manager = Arc::new(KeysManager::new(&seed, cur.as_secs(), cur.subsec_nanos()));

        let chain_monitor: Arc<ChainMonitor> = Arc::new(chainmonitor::ChainMonitor::new(
            None,
            broadcaster.clone(),
            logger.clone(),
            fee_estimator.clone(),
            persister.clone(),
        ));

        let mut channelmonitors = persister.read_channelmonitors(keys_manager.clone())?;

        // TODO: likely expose a lot of this config to our LightningNodeConfig
        let mut user_config = UserConfig::default();
        user_config
            .peer_channel_config_limits
            .force_announced_channel_preference = false;

        let best_block = chain_manager.get_best_block().await?;

        let (channel_manager_blockhash, channel_manager) = {
            if let Ok(mut f) = fs::File::open(channel_manager_path) {
                let mut channel_monitor_mut_references = Vec::new();
                for (_, channel_monitor) in channelmonitors.iter_mut() {
                    channel_monitor_mut_references.push(channel_monitor);
                }
                let read_args = ChannelManagerReadArgs::new(
                    keys_manager.clone(),
                    fee_estimator.clone(),
                    chain_monitor.clone(),
                    broadcaster.clone(),
                    logger.clone(),
                    user_config,
                    channel_monitor_mut_references,
                );
                <(BlockHash, ChannelManager)>::read(&mut f, read_args).unwrap()
            } else {
                // TODO: in reality we could error for other reasons when there's supposed to be
                // an existing chanenl manager.  need to handle this the same way we do for seed file
                // really should extract to generic error handle for io where we really want to know if
                // the file exists or not.

                let tip_hash = best_block.block_hash();
                let chain_params = ChainParameters {
                    network: config.network,
                    best_block,
                };
                let fresh_channel_manager = channelmanager::ChannelManager::new(
                    fee_estimator.clone(),
                    chain_monitor.clone(),
                    broadcaster.clone(),
                    logger.clone(),
                    keys_manager.clone(),
                    user_config,
                    chain_params,
                );
                (tip_hash, fresh_channel_manager)
            }
        };

        let mut bundled_channel_monitors = Vec::new();
        for (blockhash, channel_monitor) in channelmonitors.drain(..) {
            let outpoint = channel_monitor.get_funding_txo().0;
            bundled_channel_monitors.push((
                blockhash,
                (
                    channel_monitor,
                    broadcaster.clone(),
                    fee_estimator.clone(),
                    logger.clone(),
                ),
                outpoint,
            ));
        }

        let monitor_info = bundled_channel_monitors
            .iter_mut()
            .map(|monitor_bundle| (monitor_bundle.0, &monitor_bundle.1));

        let mut chain_listeners = vec![(
            channel_manager_blockhash,
            &channel_manager as &(dyn chain::Listen + Send + Sync),
        )];

        for (block_hash, monitor) in monitor_info {
            chain_listeners.push((block_hash, monitor as &(dyn chain::Listen + Send + Sync)));
        }

        let bdk_database_last_sync =
            node_database.find_or_create_last_sync(best_block.block_hash())?;

        chain_listeners.push((
            bdk_database_last_sync,
            &listener_database as &(dyn chain::Listen + Send + Sync),
        ));

        let tip = chain_manager
            .synchronize_to_tip(chain_listeners)
            .await
            .unwrap();

        let synced_hash = tip.header.block_hash();

        for confirmable_monitor in bundled_channel_monitors.drain(..) {
            chain_monitor
                .watch_channel(confirmable_monitor.2, confirmable_monitor.1 .0)
                .unwrap();
        }

        let channel_manager: Arc<ChannelManager> = Arc::new(channel_manager);

        // is it safe to start this now instead of in `start`
        // need to better understand separation; will depend on actual creation and startup flows
        let listener_database =
            ListenerDatabase::new(config.bdk_database_path(), config.node_database_path());
        let channel_manager_sync = channel_manager.clone();
        let chain_monitor_sync = chain_monitor.clone();

        chain_manager
            .keep_in_sync(
                synced_hash,
                channel_manager_sync,
                chain_monitor_sync,
                listener_database,
            )
            .await
            .unwrap();

        let network_graph = match network_graph {
            Some(network_graph) => network_graph,
            None => {
                let genesis = genesis_block(config.network).header.block_hash();

                Arc::new(disk::read_network(
                    Path::new(&config.network_graph_path()),
                    genesis,
                ))
            }
        };

        let network_graph_msg_handler: Arc<NetworkGraphMessageHandler> =
            match network_graph_msg_handler {
                Some(network_graph_msg_handler) => network_graph_msg_handler,
                None => Arc::new(NetworkGraphMessageHandler::new(
                    Arc::clone(&network_graph),
                    None::<Arc<dyn chain::Access + Send + Sync>>,
                    logger.clone(),
                )),
            };

        let route_handler = match config.external_router {
            true => Arc::new(OptionalNetworkGraphMsgHandler {
                network_graph_msg_handler: None,
            }),
            false => Arc::new(OptionalNetworkGraphMsgHandler {
                network_graph_msg_handler: Some(network_graph_msg_handler.clone()),
            }),
        };

        let lightning_msg_handler = MessageHandler {
            chan_handler: channel_manager.clone(),
            route_handler,
        };

        // Step 13: Initialize the PeerManager
        let mut ephemeral_bytes = [0; 32];
        rand::thread_rng().fill_bytes(&mut ephemeral_bytes);

        let peer_manager = Arc::new(PeerManager::new(
            lightning_msg_handler,
            keys_manager.get_node_secret(Recipient::Node).unwrap(),
            &ephemeral_bytes,
            logger.clone(),
            Arc::new(IgnoringMessageHandler {}),
        ));

        let scorer_path = config.scorer_path();
        let scorer = Arc::new(Mutex::new(disk::read_scorer(
            Path::new(&scorer_path),
            Arc::clone(&network_graph),
        )));

        let router = DefaultRouter::new(
            network_graph.clone(),
            logger.clone(),
            keys_manager.get_secure_random_bytes(),
        );

        let pubkey = channel_manager.get_our_node_id().to_string();

        let macaroon = LightningNode::find_or_create_macaroon(
            &seed,
            pubkey,
            admin_macaroon_path,
            &mut node_database,
        )?;

        let database = Arc::new(Mutex::new(node_database));

        let event_handler = Arc::new(LightningNodeEventHandler {
            config: config.clone(),
            wallet: bdk_wallet.clone(),
            channel_manager: channel_manager.clone(),
            keys_manager: keys_manager.clone(),
            database: database.clone(),
            tokio_handle: Handle::current(),
            chain_manager: chain_manager.clone(),
        });

        let invoice_payer = Arc::new(InvoicePayer::new(
            channel_manager.clone(),
            router,
            scorer.clone(),
            logger.clone(),
            event_handler,
            payment::RetryAttempts(5),
        ));

        let stop_listen = Arc::new(AtomicBool::new(false));

        Ok(LightningNode {
            config,
            database,
            seed,
            macaroon,
            wallet: bdk_wallet,
            channel_manager,
            chain_monitor,
            chain_manager,
            peer_manager,
            network_graph,
            network_graph_msg_handler,
            keys_manager,
            logger,
            scorer,
            invoice_payer,
            stop_listen,
        })
    }

    pub async fn start(self) -> (Vec<JoinHandle<()>>, BackgroundProcessor) {
        let mut handles = vec![];
        let config = self.config.clone();

        let peer_manager_connection_handler = self.peer_manager.clone();

        let stop_listen_ref = Arc::clone(&self.stop_listen);
        handles.push(tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(format!(
                "0.0.0.0:{}",
                self.config.ldk_peer_listening_port
            ))
            .await
            .expect("Failed to bind to listen port - is something else already listening on it?");
            loop {
                let peer_mgr = peer_manager_connection_handler.clone();
                let tcp_stream = listener.accept().await.unwrap().0;
                if stop_listen_ref.load(Ordering::Acquire) {
                    return;
                }
                tokio::spawn(async move {
                    lightning_net_tokio::setup_inbound(
                        peer_mgr.clone(),
                        tcp_stream.into_std().unwrap(),
                    )
                    .await;
                });
            }
        }));

        let scorer_path = self.config.scorer_path();
        let scorer_persist = Arc::clone(&self.scorer);

        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600));
            loop {
                interval.tick().await;
                if disk::persist_scorer(Path::new(&scorer_path), &scorer_persist.lock().unwrap())
                    .is_err()
                {
                    // Persistence errors here are non-fatal as channels will be re-scored as payments
                    // fail, but they may indicate a disk error which could be fatal elsewhere.
                    eprintln!("Warning: Failed to persist scorer, check your disk and permissions");
                }
            }
        }));

        let persister = DataPersister {
            data_dir: self.config.data_dir(),
            external_router: self.config.external_router,
        };

        // TODO: should we allow 'child' nodes to update NetworkGraph based on payment failures?
        //       feels like probably but depends on exactly what is updated
        let background_processor = BackgroundProcessor::start(
            persister,
            self.invoice_payer.clone(),
            self.chain_monitor.clone(),
            self.channel_manager.clone(),
            Some(self.network_graph_msg_handler.clone()),
            self.peer_manager.clone(),
            self.logger.clone(),
        );

        // Reconnect to channel peers if possible.

        let channel_peer_data_path = config.channel_peer_data_path();
        let channel_manager_reconnect = self.channel_manager.clone();
        let peer_manager_reconnect = self.peer_manager.clone();

        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                match disk::read_channel_peer_data(Path::new(&channel_peer_data_path)) {
                    Ok(mut info) => {
                        for (pubkey, peer_addr) in info.drain() {
                            for chan_info in channel_manager_reconnect.list_channels() {
                                if pubkey == chan_info.counterparty.node_id {
                                    let _ = connect_peer_if_necessary(
                                        pubkey,
                                        peer_addr,
                                        peer_manager_reconnect.clone(),
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                    Err(e) => println!(
                        "ERROR: errored reading channel peer info from disk: {:?}",
                        e
                    ),
                };
            }
        }));

        // Regularly broadcast our node_announcement. This is only required (or possible) if we have
        // some public channels, and is only useful if we have public listen address(es) to announce.
        // In a production environment, this should occur only after the announcement of new channels
        // to avoid churn in the global network graph.
        let chan_manager = Arc::clone(&self.channel_manager);
        let listen_addresses = config
            .ldk_announced_listen_addr
            .iter()
            .filter_map(|addr| match IpAddr::from_str(addr) {
                Ok(IpAddr::V4(a)) => Some(NetAddress::IPv4 {
                    addr: a.octets(),
                    port: config.ldk_peer_listening_port,
                }),
                Ok(IpAddr::V6(a)) => Some(NetAddress::IPv6 {
                    addr: a.octets(),
                    port: config.ldk_peer_listening_port,
                }),
                Err(_) => {
                    println!("Failed to parse announced-listen-addr into an IP address");
                    None
                }
            })
            .collect::<Vec<NetAddress>>();

        let node_alias: [u8; 32] = match config.ldk_announced_node_name {
            Some(alias) => {
                if alias.len() > 32 {
                    // TODO: probably don't panic here, this shouldn't have gotten to this state
                    panic!("alias must be less than 32 bytes");
                }
                let mut bytes = [0; 32];
                bytes[..alias.len()].copy_from_slice(alias.as_bytes());
                bytes
            }
            None => [0; 32],
        };

        handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;

                if !listen_addresses.is_empty() {
                    chan_manager.broadcast_node_announcement(
                        [0; 3],
                        node_alias,
                        listen_addresses.clone(),
                    );
                }
            }
        }));

        (handles, background_processor)
    }

    pub fn get_config(&self) -> &LightningNodeConfig {
        &self.config
    }

    // `custom_id` will be user_channel_id in FundingGenerated event
    // allows use to tie the create_channel call with the event
    pub fn open_channel(
        &self,
        peer_pubkey: PublicKey,
        channel_amt_sat: u64,
        push_amt_msat: u64,
        custom_id: u64,
        announced_channel: bool,
    ) -> Result<[u8; 32], Error> {
        let config = UserConfig {
            peer_channel_config_limits: ChannelHandshakeLimits {
                // lnd's max to_self_delay is 2016, so we want to be compatible.
                their_to_self_delay: 2016,
                ..Default::default()
            },
            channel_options: ChannelConfig {
                announced_channel,
                ..Default::default()
            },
            ..Default::default()
        };

        // TODO: want to be logging channels in db for matching forwarded payments
        match self.channel_manager.create_channel(
            peer_pubkey,
            channel_amt_sat,
            push_amt_msat,
            custom_id,
            Some(config),
        ) {
            Ok(short_channel_id) => {
                println!("EVENT: initiated channel with peer {}. ", peer_pubkey);
                Ok(short_channel_id)
            }
            Err(e) => {
                println!("ERROR: failed to open channel: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub async fn connect_to_peer(&self, pubkey: PublicKey, addr: SocketAddr) -> Result<(), Error> {
        match lightning_net_tokio::connect_outbound(Arc::clone(&self.peer_manager), pubkey, addr)
            .await
        {
            Some(connection_closed_future) => {
                let mut connection_closed_future = Box::pin(connection_closed_future);
                loop {
                    match futures::poll!(&mut connection_closed_future) {
                        std::task::Poll::Ready(_) => {
                            println!("ERROR: Peer disconnected before we finished the handshake");
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "ERROR: peer disconnected before we finished the handshake",
                            )
                            .into());
                        }
                        std::task::Poll::Pending => {}
                    }
                    // Avoid blocking the tokio context by sleeping a bit
                    match self
                        .peer_manager
                        .get_peer_node_ids()
                        .iter()
                        .find(|id| **id == pubkey)
                    {
                        Some(_) => break,
                        None => tokio::time::sleep(Duration::from_millis(10)).await,
                    }
                }
            }
            None => {
                println!("ERROR: failed to connect to peer");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "ERROR: failed to connect to peer",
                )
                .into());
            }
        }
        Ok(())
    }

    fn keysend<K: KeysInterface>(
        &self,
        invoice_payer: &InvoicePayer,
        payee_pubkey: PublicKey,
        amt_msat: u64,
        keys: &K,
    ) -> Result<(), Error> {
        let payment_preimage = keys.get_secure_random_bytes();

        let status = match invoice_payer.pay_pubkey(
            payee_pubkey,
            PaymentPreimage(payment_preimage),
            amt_msat,
            40,
        ) {
            Ok(_payment_id) => {
                println!(
                    "EVENT: initiated sending {} msats to {}",
                    amt_msat, payee_pubkey
                );
                print!("> ");
                HTLCStatus::Pending
            }
            Err(PaymentError::Invoice(e)) => {
                println!("ERROR: invalid payee: {}", e);
                print!("> ");
                return Ok(());
            }
            Err(PaymentError::Routing(e)) => {
                println!("ERROR: failed to find route: {}", e.err);
                print!("> ");
                return Ok(());
            }
            Err(PaymentError::Sending(e)) => {
                println!("ERROR: failed to send payment: {:?}", e);
                print!("> ");
                HTLCStatus::Failed
            }
        };

        let payment_hash = PaymentHash(Sha256::hash(&payment_preimage).into_inner());

        let mut database = self.database.lock().unwrap();

        database.create_or_update_payment(
            PaymentInfo {
                hash: payment_hash,
                preimage: None,
                secret: None,
                status,
                amt_msat: MillisatAmount(Some(amt_msat)),
                origin: PaymentOrigin::SpontaneousOutgoing,
                label: None,
                invoice: None,
            }
            .into(),
        )?;

        Ok(())
    }

    pub fn send_payment(&self, invoice: &Invoice) -> Result<(), Error> {
        let status = match self.invoice_payer.pay_invoice(invoice) {
            Ok(_payment_id) => {
                let payee_pubkey = invoice.recover_payee_pub_key();
                let amt_msat = invoice.amount_milli_satoshis().unwrap();
                println!(
                    "EVENT: initiated sending {} msats to {}",
                    amt_msat, payee_pubkey
                );
                HTLCStatus::Pending
            }
            Err(PaymentError::Invoice(e)) => {
                println!("ERROR: invalid invoice: {}", e);
                return Err(PaymentError::Invoice(e).into());
            }
            Err(PaymentError::Routing(e)) => {
                println!("ERROR: failed to find route: {}", e.err);
                return Err(e.into());
            }
            Err(PaymentError::Sending(e)) => {
                println!("ERROR: failed to send payment: {:?}", e);
                HTLCStatus::Failed
            }
        };
        let payment_hash = PaymentHash((*invoice.payment_hash()).into_inner());
        let payment_secret = Some(*invoice.payment_secret());

        let mut database = self.database.lock().unwrap();
        database.create_or_update_payment(
            PaymentInfo {
                hash: payment_hash,
                preimage: None,
                secret: payment_secret,
                status,
                amt_msat: MillisatAmount(invoice.amount_milli_satoshis()),
                origin: PaymentOrigin::InvoiceOutgoing,
                label: None,
                invoice: Some(invoice.to_string()),
            }
            .into(),
        )?;

        Ok(())
    }

    pub fn get_invoice(&self, amt_msat: u64, description: String) -> Result<Invoice, Error> {
        let mut database = self.database.lock().unwrap();
        let currency = match self.config.network {
            Network::Bitcoin => Currency::Bitcoin,
            Network::Testnet => Currency::BitcoinTestnet,
            Network::Regtest => Currency::Regtest,
            Network::Signet => panic!("Signet unsupported"),
        };

        let invoice = utils::create_invoice_from_channelmanager(
            &self.channel_manager,
            self.keys_manager.clone(),
            currency,
            Some(amt_msat),
            description.clone(),
        )?;

        let payment_hash = PaymentHash((*invoice.payment_hash()).into_inner());
        database.create_or_update_payment(
            PaymentInfo {
                hash: payment_hash,
                preimage: None,
                secret: Some(*invoice.payment_secret()),
                status: HTLCStatus::Pending,
                amt_msat: MillisatAmount(Some(amt_msat)),
                origin: PaymentOrigin::InvoiceIncoming,
                label: Some(description),
                invoice: Some(invoice.to_string()),
            }
            .into(),
        )?;

        Ok(invoice)
    }

    pub fn list_channels(
        &self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<Channel>, PaginationResponse), Error> {
        let query = pagination.query.unwrap_or_else(|| String::from(""));
        let per_page: usize = pagination.take.try_into().unwrap();
        let page: usize = pagination.page.try_into().unwrap();
        let index = page * per_page;

        let channels = self
            .channel_manager
            .list_channels()
            .into_iter()
            .filter_map(|chan_info| {
                let mut channel: Channel = chan_info.clone().into();

                channel.alias = self
                    .get_alias_for_channel_counterparty(&chan_info)
                    .map(|alias_bytes| hex_utils::sanitize_string(&alias_bytes));

                let match_channel = channel.clone();
                let matches_channel_id = match_channel.channel_id.contains(&query);
                let matches_pubkey = match_channel.counterparty_pubkey.contains(&query);
                let matches_funding_txid = match_channel
                    .funding_txid
                    .map(|txid| txid.contains(&query))
                    .unwrap_or(false);
                let matches_alias = match_channel
                    .alias
                    .map(|alias| alias.contains(&query))
                    .unwrap_or(false);
                let matches =
                    matches_channel_id || matches_funding_txid || matches_pubkey || matches_alias;
                if matches {
                    Some(channel)
                } else {
                    None
                }
            })
            .collect::<Vec<Channel>>();

        let paginated_channels = PagedVec::new(&channels, per_page);
        let current_page = paginated_channels
            .page(index)
            .map(|channels_page| channels_page.1.to_vec())
            .unwrap_or_default();
        let next_page = paginated_channels.page(index + per_page);

        let pagination_response = PaginationResponse {
            has_more: next_page.is_some(),
            total: channels.len() as u64,
        };
        Ok((current_page, pagination_response))
    }

    pub fn list_transactions(
        &self,
        pagination: PaginationRequest,
    ) -> Result<(Vec<TransactionDetails>, PaginationResponse), Error> {
        let query = pagination.query.unwrap_or_else(|| String::from(""));
        let per_page: usize = pagination.take.try_into().unwrap();
        let page: usize = pagination.page.try_into().unwrap();
        let index = page * per_page;

        let bdk_wallet = self.wallet.lock().unwrap();

        let transaction_details = bdk_wallet
            .list_transactions(false)?
            .into_iter()
            .filter(|tx_details| {
                let match_transaction_details = tx_details.clone();
                match_transaction_details.txid.to_string().contains(&query)
            })
            .collect::<Vec<TransactionDetails>>();

        let paginated_transactions = PagedVec::new(&transaction_details, per_page);
        let current_page = paginated_transactions
            .page(index)
            .map(|transactions_page| transactions_page.1.to_vec())
            .unwrap_or_default();
        let next_page = paginated_transactions.page(index + per_page);

        let pagination_response = PaginationResponse {
            has_more: next_page.is_some(),
            total: transaction_details.len() as u64,
        };
        Ok((current_page, pagination_response))
    }

    pub fn get_alias_for_channel_counterparty(
        &self,
        channel_details: &ChannelDetails,
    ) -> Option<[u8; 32]> {
        let node_id = NodeId::from_pubkey(&channel_details.counterparty.node_id);

        let alias = self
            .network_graph
            .read_only()
            .nodes()
            .get(&node_id)
            .and_then(|node_info| {
                node_info
                    .announcement_info
                    .clone()
                    .map(|ann_info| ann_info.alias)
            });

        alias
    }

    pub fn list_payments(
        &self,
        pagination: PaginationRequest,
        filter: PaymentsFilter,
    ) -> Result<(Vec<database::node::Payment>, PaginationResponse), Error> {
        let database = self.database.lock().unwrap();
        database.get_payments(pagination, filter).map_err(Error::Db)
    }

    pub fn close_channel(&self, channel_id: [u8; 32], force: bool) -> Result<(), Error> {
        if force {
            Ok(self.channel_manager.force_close_channel(&channel_id)?)
        } else {
            Ok(self.channel_manager.close_channel(&channel_id)?)
        }
    }

    pub fn node_info(&self) -> Result<NodeInfo, Error> {
        let chans = self.channel_manager.list_channels();
        let local_balance_msat = chans.iter().map(|c| c.balance_msat).sum::<u64>();

        Ok(NodeInfo {
            node_pubkey: self.channel_manager.get_our_node_id().to_string(),
            num_channels: chans.len() as u32,
            num_usable_channels: chans.iter().filter(|c| c.is_usable).count() as u32,
            num_peers: self.peer_manager.get_peer_node_ids().len() as u32,
            local_balance_msat,
        })
    }

    pub fn get_invoice_from_str(&self, invoice: &str) -> Result<Invoice, Error> {
        Ok(Invoice::from_str(invoice)?)
    }

    pub fn list_peers(&self) -> Result<Vec<Peer>, Error> {
        let peers = self
            .peer_manager
            .get_peer_node_ids()
            .iter()
            .map(|pubkey| Peer {
                node_pubkey: format!("{}", pubkey),
            })
            .collect::<Vec<Peer>>();
        Ok(peers)
    }

    pub fn sign_message(&self, message: String) -> Result<String, Error> {
        Ok(lightning::util::message_signing::sign(
            message.as_bytes(),
            &self.keys_manager.get_node_secret(Recipient::Node).unwrap(),
        )?)
    }

    pub async fn delete_payment(&self, payment_hash: String) -> Result<(), Error> {
        let database = self.database.lock().unwrap();
        database.delete_payment(payment_hash)?;
        Ok(())
    }

    pub async fn label_payment(&self, label: String, payment_hash: String) -> Result<(), Error> {
        let mut database = self.database.lock().unwrap();
        let payment = database.get_payment(payment_hash)?;
        if let Some(mut payment) = payment {
            payment.label = Some(label);
            database.create_or_update_payment(payment)?;
        }

        Ok(())
    }

    pub async fn call(&self, request: NodeRequest) -> Result<NodeResponse, NodeRequestError> {
        match request {
            NodeRequest::StartNode { passphrase: _ } => Ok(NodeResponse::StartNode {}),
            NodeRequest::StopNode {} => Ok(NodeResponse::StopNode {}),
            NodeRequest::GetUnusedAddress {} => {
                let wallet = self.wallet.lock().unwrap();
                let address_info = wallet.get_address(AddressIndex::LastUnused)?;
                Ok(NodeResponse::GetUnusedAddress {
                    address: address_info.address.to_string(),
                })
            }
            NodeRequest::GetBalance {} => {
                let wallet = self.wallet.lock().unwrap();
                let balance = wallet.get_balance().map_err(Error::Bdk)?;
                Ok(NodeResponse::GetBalance {
                    balance_satoshis: balance,
                })
            }
            NodeRequest::OpenChannel {
                node_connection_string,
                amt_satoshis,
                public,
            } => {
                let (pubkey, addr) = parse_peer_info(node_connection_string.clone())?;

                let found_peer = self
                    .peer_manager
                    .get_peer_node_ids()
                    .into_iter()
                    .find(|node_pubkey| *node_pubkey == pubkey);

                if found_peer.is_none() {
                    self.connect_to_peer(pubkey, addr).await?;
                }

                let res = self.open_channel(pubkey, amt_satoshis, 0, 0, public);

                if res.is_ok() {
                    let _ = disk::persist_channel_peer(
                        Path::new(&self.config.channel_peer_data_path()),
                        &node_connection_string,
                    );
                }

                Ok(NodeResponse::OpenChannel {})
            }
            NodeRequest::SendPayment { invoice } => {
                let invoice = self.get_invoice_from_str(&invoice)?;
                self.send_payment(&invoice)?;
                Ok(NodeResponse::SendPayment {})
            }
            NodeRequest::Keysend {
                dest_pubkey,
                amt_msat,
            } => match hex_utils::to_compressed_pubkey(&dest_pubkey) {
                Some(pubkey) => {
                    self.keysend(&*self.invoice_payer, pubkey, amt_msat, &*self.keys_manager)?;
                    Ok(NodeResponse::Keysend {})
                }
                None => Err(NodeRequestError::Sensei("invalid dest_pubkey".into())),
            },
            NodeRequest::GetInvoice {
                amt_msat,
                description,
            } => {
                let invoice = self.get_invoice(amt_msat, description)?;
                let invoice_str = format!("{}", invoice);
                Ok(NodeResponse::GetInvoice {
                    invoice: invoice_str,
                })
            }
            NodeRequest::LabelPayment {
                label,
                payment_hash,
            } => {
                self.label_payment(label, payment_hash).await?;
                Ok(NodeResponse::LabelPayment {})
            }
            NodeRequest::DeletePayment { payment_hash } => {
                self.delete_payment(payment_hash).await?;
                Ok(NodeResponse::DeletePayment {})
            }
            NodeRequest::ConnectPeer {
                node_connection_string,
            } => {
                let (pubkey, addr) = parse_peer_info(node_connection_string)?;

                let found_peer = self
                    .peer_manager
                    .get_peer_node_ids()
                    .into_iter()
                    .find(|node_pubkey| *node_pubkey == pubkey);

                if found_peer.is_none() {
                    self.connect_to_peer(pubkey, addr).await?;
                }

                Ok(NodeResponse::ConnectPeer {})
            }
            NodeRequest::ListChannels { pagination } => {
                let (channels, pagination) = self.list_channels(pagination)?;
                Ok(NodeResponse::ListChannels {
                    channels,
                    pagination,
                })
            }
            NodeRequest::ListTransactions { pagination } => {
                let (transactions, pagination) = self.list_transactions(pagination)?;
                Ok(NodeResponse::ListTransactions {
                    transactions,
                    pagination,
                })
            }
            NodeRequest::ListPayments { pagination, filter } => {
                let (payments, pagination) = self.list_payments(pagination, filter)?;
                Ok(NodeResponse::ListPayments {
                    payments,
                    pagination,
                })
            }
            NodeRequest::CloseChannel { channel_id, force } => {
                let mut channel_id_bytes = [0u8; 32];
                let bytes = hex_utils::to_vec(&channel_id);
                if let Some(bytes) = bytes {
                    channel_id_bytes.copy_from_slice(&bytes)
                }
                self.close_channel(channel_id_bytes, force)?;
                Ok(NodeResponse::CloseChannel {})
            }
            NodeRequest::NodeInfo {} => {
                let node_info = self.node_info()?;
                Ok(NodeResponse::NodeInfo { node_info })
            }
            NodeRequest::ListPeers {} => {
                let peers = self.list_peers()?;
                Ok(NodeResponse::ListPeers { peers })
            }
            NodeRequest::SignMessage { message } => {
                let signature = self.sign_message(message)?;
                Ok(NodeResponse::SignMessage { signature })
            }
        }
    }
}

pub fn parse_peer_info(
    peer_pubkey_and_ip_addr: String,
) -> Result<(PublicKey, SocketAddr), std::io::Error> {
    let mut pubkey_and_addr = peer_pubkey_and_ip_addr.split('@');
    let pubkey = pubkey_and_addr.next();
    let peer_addr_str = pubkey_and_addr.next();
    if peer_addr_str.is_none() || peer_addr_str.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: incorrectly formatted peer info. Should be formatted as: `pubkey@host:port`",
        ));
    }

    let peer_addr = peer_addr_str
        .unwrap()
        .to_socket_addrs()
        .map(|mut r| r.next());
    if peer_addr.is_err() || peer_addr.as_ref().unwrap().is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: couldn't parse pubkey@host:port into a socket address",
        ));
    }

    let pubkey = hex_utils::to_compressed_pubkey(pubkey.unwrap());
    if pubkey.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: unable to parse given pubkey for node",
        ));
    }

    Ok((pubkey.unwrap(), peer_addr.unwrap().unwrap()))
}

pub(crate) async fn connect_peer_if_necessary(
    pubkey: PublicKey,
    peer_addr: SocketAddr,
    peer_manager: Arc<PeerManager>,
) -> Result<(), ()> {
    for node_pubkey in peer_manager.get_peer_node_ids() {
        if node_pubkey == pubkey {
            return Ok(());
        }
    }
    match lightning_net_tokio::connect_outbound(Arc::clone(&peer_manager), pubkey, peer_addr).await
    {
        Some(connection_closed_future) => {
            let mut connection_closed_future = Box::pin(connection_closed_future);
            loop {
                match futures::poll!(&mut connection_closed_future) {
                    std::task::Poll::Ready(_) => {
                        println!("ERROR: Peer disconnected before we finished the handshake");
                        return Err(());
                    }
                    std::task::Poll::Pending => {}
                }
                // Avoid blocking the tokio context by sleeping a bit
                match peer_manager
                    .get_peer_node_ids()
                    .iter()
                    .find(|id| **id == pubkey)
                {
                    Some(_) => break,
                    None => tokio::time::sleep(Duration::from_millis(10)).await,
                }
            }
        }
        None => {
            //println!("ERROR: failed to connect to peer");
            return Err(());
        }
    }
    Ok(())
}
