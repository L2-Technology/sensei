// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::chain::broadcaster::SenseiBroadcaster;
use crate::chain::database::WalletDatabase;
use crate::chain::fee_estimator::SenseiFeeEstimator;
use crate::chain::manager::SenseiChainManager;
use crate::channels::ChannelOpener;
use crate::config::SenseiConfig;
use crate::database::SenseiDatabase;
use crate::disk::FilesystemLogger;
use crate::error::Error;
use crate::event_handler::LightningNodeEventHandler;
use crate::events::SenseiEvent;
use crate::p2p::bubble_gossip_route_handler::{AnyP2PGossipHandler, BubbleGossipRouteHandler};
use crate::p2p::router::{AnyRouter, AnyScorer};
use crate::p2p::utils::parse_peer_info;
use crate::p2p::SenseiP2P;
use crate::persist::{AnyKVStore, DatabaseStore, SenseiPersister};
use crate::services::node::{
    Channel, NodeInfo, NodeRequest, NodeRequestError, NodeResponse, OpenChannelRequest,
    OpenChannelResult, Peer, Utxo,
};
use crate::services::{PaginationRequest, PaginationResponse, PaymentsFilter};
use crate::utils::PagedVec;
use crate::{hex_utils, version};
use bdk::keys::ExtendedKey;
use bdk::wallet::time;
use bdk::wallet::AddressIndex;
use bdk::TransactionDetails;
use bitcoin::hashes::Hash;
use entity::sea_orm::{ActiveModelTrait, ActiveValue};
use lightning::chain::channelmonitor::ChannelMonitor;

use lightning::ln::features::InvoiceFeatures;
use lightning::ln::msgs::NetAddress;
use lightning_invoice::payment::PaymentError;
use tindercrypt::cryptors::RingCryptor;

use bdk::template::DescriptorTemplateOut;
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::{PublicKey, Secp256k1};
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use bitcoin::BlockHash;
use lightning::chain::chainmonitor;
use lightning::chain::keysinterface::{
    InMemorySigner, KeysInterface, KeysManager, PhantomKeysManager, Recipient,
};
use lightning::chain::Watch;
use lightning::chain::{self, Filter};
use lightning::ln::channelmanager::{
    self, ChannelDetails, ChannelManager as LdkChannelManager, PhantomRouteHints,
};
use lightning::ln::channelmanager::{ChainParameters, ChannelManagerReadArgs};
use lightning::ln::peer_handler::{
    ErroringMessageHandler, IgnoringMessageHandler, MessageHandler, PeerManager as LdkPeerManager,
};
use lightning::ln::{PaymentHash, PaymentPreimage, PaymentSecret};
use lightning::routing::gossip::{
    NetworkGraph as LdkNetworkGraph, NodeId, P2PGossipSync, RoutingFees,
};
use lightning::routing::router::{RouteHint, RouteHintHop};
use lightning::routing::scoring::ProbabilisticScorer;
use lightning::util::config::UserConfig;
use lightning::util::ser::{Readable, ReadableArgs, Writeable};
use lightning_background_processor::BackgroundProcessor;
use lightning_invoice::utils::DefaultRouter;
use lightning_invoice::{payment, utils, Currency, Invoice, InvoiceDescription};
use lightning_net_tokio::SocketDescriptor;
use lightning_rapid_gossip_sync::RapidGossipSync;
use macaroon::Macaroon;
use rand::{thread_rng, RngCore};
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use std::fmt::Display;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{convert::From, fmt};
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

#[derive(Serialize, Debug)]
pub struct LocalInvoice {
    pub payment_hash: String,
    pub currency: String,
    pub amount: u64,
    pub description: String,
    pub expiry: u64,
    pub timestamp: u64,
    pub min_final_cltv_expiry: u64,
    #[serde(serialize_with = "serialize_route_hints")]
    pub route_hints: Vec<RouteHint>,
    pub features: Option<LocalInvoiceFeatures>,
    pub payee_pub_key: PublicKey,
}

fn serialize_route_hints<S>(vector: &Vec<RouteHint>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vector.len()))?;
    for item in vector {
        let local_hint: LocalRouteHint = item.into();
        seq.serialize_element(&local_hint)?;
    }
    seq.end()
}

impl From<Invoice> for LocalInvoice {
    fn from(invoice: Invoice) -> Self {
        Self {
            payment_hash: invoice.payment_hash().to_string(),
            currency: invoice.currency().to_string(),
            amount: invoice.amount_milli_satoshis().unwrap_or_default(),
            description: match invoice.description() {
                InvoiceDescription::Direct(description) => description.clone().into_inner(),
                _ => String::from(""),
            },
            expiry: invoice.expiry_time().as_secs(),
            timestamp: invoice
                .timestamp()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            min_final_cltv_expiry: invoice.min_final_cltv_expiry(),
            route_hints: invoice.route_hints(),
            features: invoice.features().map(|f| f.into()),
            payee_pub_key: invoice.recover_payee_pub_key(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct LocalRouteHint {
    #[serde(serialize_with = "serialize_route_hint_hops")]
    pub hops: Vec<RouteHintHop>,
}

fn serialize_route_hint_hops<S>(
    vector: &Vec<RouteHintHop>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(vector.len()))?;
    for item in vector {
        let local_hint: LocalRouteHintHop = item.into();
        seq.serialize_element(&local_hint)?;
    }
    seq.end()
}

impl From<&RouteHint> for LocalRouteHint {
    fn from(hint: &RouteHint) -> Self {
        Self {
            hops: hint.0.clone(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct LocalRouteHintHop {
    pub src_node_id: PublicKey,
    pub short_channel_id: u64,
    #[serde(with = "LocalRoutingFees")]
    pub fees: RoutingFees,
    pub cltv_expiry_delta: u16,
    pub htlc_minimum_msat: Option<u64>,
    pub htlc_maximum_msat: Option<u64>,
}

impl From<&RouteHintHop> for LocalRouteHintHop {
    fn from(hop: &RouteHintHop) -> Self {
        Self {
            src_node_id: hop.src_node_id,
            short_channel_id: hop.short_channel_id,
            fees: hop.fees,
            cltv_expiry_delta: hop.cltv_expiry_delta,
            htlc_minimum_msat: hop.htlc_minimum_msat,
            htlc_maximum_msat: hop.htlc_maximum_msat,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(remote = "RoutingFees")]
pub struct LocalRoutingFees {
    pub base_msat: u32,
    pub proportional_millionths: u32,
}

impl From<RoutingFees> for LocalRoutingFees {
    fn from(fees: RoutingFees) -> Self {
        Self {
            base_msat: fees.base_msat,
            proportional_millionths: fees.proportional_millionths,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct LocalInvoiceFeatures {
    pub variable_length_onion: bool,
    pub payment_secret: bool,
    pub basic_mpp: bool,
}

impl From<&InvoiceFeatures> for LocalInvoiceFeatures {
    fn from(features: &InvoiceFeatures) -> Self {
        Self {
            variable_length_onion: features.supports_variable_length_onion(),
            payment_secret: features.supports_payment_secret(),
            basic_mpp: features.supports_basic_mpp(),
        }
    }
}

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

pub type NetworkGraph = LdkNetworkGraph<Arc<FilesystemLogger>>;

pub type GossipSync<P, G, A, L> =
    lightning_background_processor::GossipSync<P, Arc<RapidGossipSync<G, L>>, G, A, L>;

pub type ChainMonitor = chainmonitor::ChainMonitor<
    InMemorySigner,
    Arc<dyn Filter + Send + Sync>,
    Arc<SenseiBroadcaster>,
    Arc<SenseiFeeEstimator>,
    Arc<FilesystemLogger>,
    Arc<SenseiPersister>,
>;

trait MustSized: Sized {}

pub type SimpleArcChannelManager<M, T, F, L> =
    LdkChannelManager<InMemorySigner, Arc<M>, Arc<T>, Arc<PhantomKeysManager>, Arc<F>, Arc<L>>;

pub type SimpleArcPeerManager<SD, M, T, F, L> = LdkPeerManager<
    SD,
    Arc<SimpleArcChannelManager<M, T, F, L>>,
    Arc<BubbleGossipRouteHandler>,
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

pub type SimpleArcRoutingPeerManager<SD, L> = LdkPeerManager<
    SD,
    Arc<ErroringMessageHandler>,
    Arc<AnyP2PGossipHandler>,
    Arc<L>,
    Arc<IgnoringMessageHandler>,
>;
pub type RoutingPeerManager = SimpleArcRoutingPeerManager<SocketDescriptor, FilesystemLogger>;

pub type ChannelManager =
    SimpleArcChannelManager<ChainMonitor, SenseiBroadcaster, SenseiFeeEstimator, FilesystemLogger>;

pub type Router = DefaultRouter<Arc<NetworkGraph>, Arc<FilesystemLogger>>;
pub type Scorer = ProbabilisticScorer<Arc<NetworkGraph>, Arc<FilesystemLogger>>;

pub type InvoicePayer = payment::InvoicePayer<
    Arc<ChannelManager>,
    AnyRouter,
    Arc<Mutex<AnyScorer>>,
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

pub type NetworkGraphMessageHandler =
    P2PGossipSync<Arc<NetworkGraph>, Arc<dyn chain::Access + Send + Sync>, Arc<FilesystemLogger>>;

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
    pub id: String,
    pub pubkey: String,
    pub scope: String,
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
    pub config: Arc<SenseiConfig>,
    pub id: String,
    pub listen_port: u16,
    pub listen_addresses: Vec<String>,
    pub seed: [u8; 32],
    pub database: Arc<SenseiDatabase>,
    pub wallet: Arc<Mutex<bdk::Wallet<WalletDatabase>>>,
    pub channel_manager: Arc<ChannelManager>,
    pub chain_monitor: Arc<ChainMonitor>,
    pub chain_manager: Arc<SenseiChainManager>,
    pub peer_manager: Arc<PeerManager>,
    pub p2p: Arc<SenseiP2P>,
    pub keys_manager: Arc<PhantomKeysManager>,
    pub logger: Arc<FilesystemLogger>,
    pub invoice_payer: Arc<InvoicePayer>,
    pub stop_listen: Arc<AtomicBool>,
    pub persister: Arc<SenseiPersister>,
    pub event_sender: broadcast::Sender<SenseiEvent>,
    pub broadcaster: Arc<SenseiBroadcaster>,
}

impl LightningNode {
    pub fn generate_entropy() -> [u8; 32] {
        let mut entropy: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut entropy);
        entropy
    }

    pub fn encrypt_entropy(entropy: &[u8; 32], passphrase: &[u8]) -> Result<Vec<u8>, Error> {
        let cryptor = RingCryptor::new();
        Ok(cryptor.seal_with_passphrase(passphrase, entropy)?)
    }

    async fn get_entropy_for_node(
        node_id: String,
        passphrase: String,
        database: Arc<SenseiDatabase>,
    ) -> Result<[u8; 32], Error> {
        let cryptor = RingCryptor::new();
        let mut entropy: [u8; 32] = [0; 32];
        match database.get_entropy(node_id.clone()).await? {
            Some(encrypted_entropy) => {
                let decrypted_entropy =
                    cryptor.open(passphrase.as_bytes(), encrypted_entropy.as_slice())?;

                if decrypted_entropy.len() != 32 {
                    return Err(Error::InvalidEntropyLength);
                }
                entropy.copy_from_slice(decrypted_entropy.as_slice());
                Ok(entropy)
            }
            None => Err(Error::EntropyNotFound),
        }
    }

    async fn get_cross_node_entropy_for_node(
        node_id: String,
        passphrase: String,
        database: Arc<SenseiDatabase>,
    ) -> Result<[u8; 32], Error> {
        let cryptor = RingCryptor::new();
        let mut entropy: [u8; 32] = [0; 32];
        match database.get_cross_node_entropy(node_id.clone()).await? {
            Some(encrypted_entropy) => {
                let decrypted_entropy =
                    cryptor.open(passphrase.as_bytes(), encrypted_entropy.as_slice())?;

                if decrypted_entropy.len() != 32 {
                    return Err(Error::InvalidEntropyLength);
                }
                entropy.copy_from_slice(decrypted_entropy.as_slice());
                Ok(entropy)
            }
            None => Err(Error::EntropyNotFound),
        }
    }

    pub fn generate_macaroon(
        seed: &[u8],
        pubkey: String,
        scope: String,
    ) -> Result<(Macaroon, String), Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let macaroon_data = MacaroonSession {
            id: id.clone(),
            pubkey,
            scope,
        };
        let serialized_macaroon_data = serde_json::to_string(&macaroon_data).unwrap();
        let macaroon_key = macaroon::MacaroonKey::from(seed);
        let macaroon_identifier = macaroon::ByteString::from(serialized_macaroon_data);
        let macaroon = macaroon::Macaroon::create(
            Some("senseid".to_string()),
            &macaroon_key,
            macaroon_identifier,
        )?;

        Ok((macaroon, id))
    }

    pub fn write_macaroon_to_file(macaroon_path: String, macaroon: &Macaroon) -> Result<(), Error> {
        let macaroon_as_bytes = macaroon.serialize(macaroon::Format::V2)?;
        match File::create(macaroon_path.clone()) {
            Ok(mut f) => {
                f.write_all(macaroon_as_bytes.as_slice())?;
                f.sync_all()?;
            }
            Err(e) => {
                println!(
                    "ERROR: Unable to create admin.macaroon file {}: {}",
                    macaroon_path, e
                );
            }
        }

        Ok(())
    }

    pub fn encrypt_macaroon(macaroon: &Macaroon, passphrase: &[u8]) -> Result<Vec<u8>, Error> {
        let cryptor = RingCryptor::new();
        let macaroon_as_bytes = macaroon.serialize(macaroon::Format::V2)?;
        let encrypted_macaroon = cryptor.seal_with_passphrase(passphrase, &macaroon_as_bytes)?;
        Ok(encrypted_macaroon)
    }

    pub async fn get_macaroon_for_node(
        node_id: &str,
        passphrase: &str,
        database: Arc<SenseiDatabase>,
    ) -> Result<Macaroon, Error> {
        let cryptor = RingCryptor::new();

        match database.get_macaroon(node_id).await? {
            Some(macaroon) => {
                let decrypted_macaroon = cryptor.open(
                    passphrase.as_bytes(),
                    macaroon.encrypted_macaroon.as_slice(),
                )?;
                let macaroon = macaroon::Macaroon::deserialize(decrypted_macaroon.as_slice())?;
                Ok(macaroon)
            }
            None => Err(Error::MacaroonNotFound),
        }
    }

    pub async fn verify_macaroon(
        &self,
        macaroon: Macaroon,
        session: MacaroonSession,
    ) -> Result<(), Error> {
        let existing_macaroon = self.database.find_macaroon_by_id(session.id).await?;

        if existing_macaroon.is_none() {
            return Err(Error::InvalidMacaroon);
        }

        let verifier = macaroon::Verifier::default();
        let key = macaroon::MacaroonKey::from(&self.seed[..]);
        verifier
            .verify(&macaroon, &key, vec![])
            .map_err(|_e| Error::InvalidMacaroon)
    }

    pub fn get_seed_from_entropy(network: Network, entropy: &[u8; 32]) -> [u8; 32] {
        let xprivkey = ExtendedPrivKey::new_master(network, entropy).unwrap();
        xprivkey.private_key.secret_bytes()
    }

    pub fn get_node_pubkey_from_seed(seed: &[u8; 32]) -> String {
        let secp_ctx = Secp256k1::new();
        let keys_manager = KeysManager::new(seed, 0, 0);
        let node_secret = keys_manager.get_node_secret(Recipient::Node).unwrap();
        let node_pubkey = PublicKey::from_secret_key(&secp_ctx, &node_secret);
        node_pubkey.to_string()
    }

    pub fn get_phantom_node_pubkey(&self) -> String {
        let secp_ctx = Secp256k1::new();
        let node_secret = self
            .keys_manager
            .get_node_secret(Recipient::PhantomNode)
            .unwrap();
        let node_pubkey = PublicKey::from_secret_key(&secp_ctx, &node_secret);
        node_pubkey.to_string()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        config: Arc<SenseiConfig>,
        id: String,
        listen_addresses: Vec<String>,
        listen_port: u16,
        alias: String,
        data_dir: String,
        passphrase: String,
        p2p: Arc<SenseiP2P>,
        chain_manager: Arc<SenseiChainManager>,
        database: Arc<SenseiDatabase>,
        event_sender: broadcast::Sender<SenseiEvent>,
    ) -> Result<(Self, Vec<JoinHandle<()>>, BackgroundProcessor), Error> {
        let network = config.network;

        let entropy =
            LightningNode::get_entropy_for_node(id.clone(), passphrase.clone(), database.clone())
                .await?;

        let cross_node_entropy = LightningNode::get_cross_node_entropy_for_node(
            id.clone(),
            passphrase.clone(),
            database.clone(),
        )
        .await?;

        let xprivkey = ExtendedPrivKey::new_master(network, &entropy).unwrap();
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

        let bdk_database = WalletDatabase::new(id.clone(), database.clone(), database.get_handle());
        let wallet_database = bdk_database.clone();

        let bdk_wallet = bdk::Wallet::new(
            receive_descriptor_template,
            Some(change_descriptor_template),
            network,
            bdk_database,
        )?;
        // TODO: probably can do this later, assuming this is REALLY slow
        bdk_wallet.ensure_addresses_cached(100).unwrap();

        let bdk_wallet = Arc::new(Mutex::new(bdk_wallet));

        let logger = Arc::new(FilesystemLogger::new(data_dir.clone(), config.network));

        let seed = LightningNode::get_seed_from_entropy(network, &entropy);
        let cross_node_seed = LightningNode::get_seed_from_entropy(network, &cross_node_entropy);

        let cur = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let keys_manager = Arc::new(PhantomKeysManager::new(
            &seed,
            cur.as_secs(),
            cur.subsec_nanos(),
            &cross_node_seed,
        ));

        let broadcaster = Arc::new(SenseiBroadcaster::new(
            id.clone(),
            chain_manager.broadcaster.clone(),
            Arc::new(Mutex::new(wallet_database.clone())),
            event_sender.clone(),
        ));

        let persistence_store =
            AnyKVStore::Database(DatabaseStore::new(database.clone(), id.clone()));

        let persister = Arc::new(SenseiPersister::new(
            persistence_store,
            config.network,
            logger.clone(),
        ));

        let chain_monitor: Arc<ChainMonitor> = Arc::new(chainmonitor::ChainMonitor::new(
            None,
            broadcaster.clone(),
            logger.clone(),
            chain_manager.fee_estimator.clone(),
            persister.clone(),
        ));

        let mut channelmonitors = persister.read_channelmonitors(keys_manager.clone())?;

        // TODO: likely expose a lot of this config to our LightningNodeConfig
        let mut user_config = UserConfig::default();

        user_config
            .channel_handshake_limits
            .force_announced_channel_preference = false;
        user_config.manually_accept_inbound_channels = true;

        let best_block = chain_manager.get_best_block().await?;

        let (channel_manager_blockhash, channel_manager) = {
            if let Ok(Some(contents)) = persister.read_channel_manager() {
                let mut channel_monitor_mut_references = Vec::new();
                for (_, channel_monitor) in channelmonitors.iter_mut() {
                    channel_monitor_mut_references.push(channel_monitor);
                }
                let read_args = ChannelManagerReadArgs::new(
                    keys_manager.clone(),
                    chain_manager.fee_estimator.clone(),
                    chain_monitor.clone(),
                    broadcaster.clone(),
                    logger.clone(),
                    user_config,
                    channel_monitor_mut_references,
                );
                let mut buffer = Cursor::new(&contents);
                <(BlockHash, ChannelManager)>::read(&mut buffer, read_args).unwrap()
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
                    chain_manager.fee_estimator.clone(),
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
                    chain_manager.fee_estimator.clone(),
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

        let onchain_wallet_sync = {
            database
                .create_or_update_last_onchain_wallet_sync(
                    id.clone(),
                    best_block.block_hash(),
                    best_block.height(),
                    time::get_timestamp(),
                )
                .await?
        };

        chain_listeners.push((
            onchain_wallet_sync.hash,
            &wallet_database as &(dyn chain::Listen + Send + Sync),
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

        let channel_manager_sync = channel_manager.clone();
        let chain_monitor_sync = chain_monitor.clone();

        chain_manager
            .keep_in_sync(
                synced_hash,
                channel_manager_sync,
                chain_monitor_sync,
                wallet_database.clone(),
            )
            .await
            .unwrap();

        let lightning_msg_handler = MessageHandler {
            chan_handler: channel_manager.clone(),
            route_handler: Arc::new(BubbleGossipRouteHandler {
                target: p2p.p2p_gossip.clone(),
            }),
        };

        let mut ephemeral_bytes = [0; 32];
        rand::thread_rng().fill_bytes(&mut ephemeral_bytes);

        let peer_manager = Arc::new(PeerManager::new(
            lightning_msg_handler,
            keys_manager.get_node_secret(Recipient::Node).unwrap(),
            &ephemeral_bytes,
            logger.clone(),
            Arc::new(IgnoringMessageHandler {}),
        ));

        let event_handler = Arc::new(LightningNodeEventHandler {
            node_id: id.clone(),
            config: config.clone(),
            wallet: bdk_wallet.clone(),
            channel_manager: channel_manager.clone(),
            keys_manager: keys_manager.clone(),
            database: database.clone(),
            tokio_handle: Handle::current(),
            chain_manager: chain_manager.clone(),
            event_sender: event_sender.clone(),
            broadcaster: broadcaster.clone(),
            secp_ctx: Secp256k1::new(),
        });

        let invoice_payer = Arc::new(InvoicePayer::new(
            channel_manager.clone(),
            p2p.get_router(),
            p2p.scorer.clone(),
            logger.clone(),
            event_handler,
            payment::Retry::Attempts(5),
        ));

        let stop_listen = Arc::new(AtomicBool::new(false));

        let mut handles = vec![];

        let peer_manager_connection_handler = peer_manager.clone();

        let stop_listen_ref = Arc::clone(&stop_listen);
        handles.push(tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", listen_port))
                .await
                .expect(
                    "Failed to bind to listen port - is something else already listening on it?",
                );
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

        let bg_persister = Arc::clone(&persister);

        // TODO: https://github.com/lightningdevkit/rust-lightning/issues/1595
        // once this lands we should be able to build a tokio BP to prevent starting
        // a thread per node. might even be able to simplify to a single BP per sensei instance
        let background_processor = BackgroundProcessor::start(
            bg_persister,
            invoice_payer.clone(),
            chain_monitor.clone(),
            channel_manager.clone(),
            lightning_background_processor::GossipSync::None::<
                Arc<
                    P2PGossipSync<
                        Arc<NetworkGraph>,
                        Arc<dyn chain::Access + Send + Sync>,
                        Arc<FilesystemLogger>,
                    >,
                >,
                Arc<RapidGossipSync<Arc<NetworkGraph>, Arc<FilesystemLogger>>>,
                Arc<NetworkGraph>,
                Arc<dyn chain::Access + Send + Sync>,
                Arc<FilesystemLogger>,
            >,
            peer_manager.clone(),
            logger.clone(),
            None::<Arc<Mutex<AnyScorer>>>,
        );

        // Reconnect to channel peers if possible.
        p2p.peer_connector
            .register_node(id.clone(), peer_manager.clone(), channel_manager.clone());

        // Regularly broadcast our node_announcement. This is only required (or possible) if we have
        // some public channels, and is only useful if we have public listen address(es) to announce.
        // In a production environment, this should occur only after the announcement of new channels
        // to avoid churn in the global network graph.
        let broadcast_listen_addresses = listen_addresses
            .iter()
            .filter_map(|addr| match IpAddr::from_str(addr) {
                Ok(IpAddr::V4(a)) => Some(NetAddress::IPv4 {
                    addr: a.octets(),
                    port: listen_port,
                }),
                Ok(IpAddr::V6(a)) => Some(NetAddress::IPv6 {
                    addr: a.octets(),
                    port: listen_port,
                }),
                Err(_) => {
                    println!("Failed to parse announced-listen-addr into an IP address");
                    None
                }
            })
            .collect::<Vec<NetAddress>>();

        let mut alias_bytes = [0; 32];
        alias_bytes[..alias.len()].copy_from_slice(alias.as_bytes());

        p2p.node_announcer.register_node(
            id.clone(),
            channel_manager.clone(),
            broadcast_listen_addresses,
            alias_bytes,
        );

        let lightning_node = LightningNode {
            config,
            id,
            listen_port,
            listen_addresses,
            database,
            seed,
            wallet: bdk_wallet,
            channel_manager,
            chain_monitor,
            chain_manager,
            peer_manager,
            p2p: p2p.clone(),
            keys_manager,
            logger,
            invoice_payer,
            stop_listen,
            persister,
            event_sender,
            broadcaster,
        };

        Ok((lightning_node, handles, background_processor))
    }

    pub async fn open_channels(
        &self,
        requests: Vec<OpenChannelRequest>,
    ) -> Result<Vec<(OpenChannelRequest, Result<[u8; 32], Error>)>, Error> {
        let mut opener = ChannelOpener::new(
            self.id.clone(),
            self.channel_manager.clone(),
            self.chain_manager.clone(),
            self.wallet.clone(),
            self.event_sender.subscribe(),
            self.broadcaster.clone(),
            self.peer_manager.clone(),
            self.p2p.peer_connector.clone(),
        );
        opener.open_batch(requests).await
    }

    // `custom_id` will be user_channel_id in FundingGenerated event
    // allows use to tie the create_channel call with the event
    pub async fn open_channel(&self, request: OpenChannelRequest) -> Result<[u8; 32], Error> {
        let requests = vec![request];
        let mut responses = self.open_channels(requests).await?;
        let (_request, result) = responses.pop().unwrap();
        result
    }

    async fn keysend<K: KeysInterface>(
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

        let payment_hash = hex_utils::hex_str(&Sha256::hash(&payment_preimage).into_inner());
        let preimage = Some(hex_utils::hex_str(&payment_preimage));

        let payment = entity::payment::ActiveModel {
            node_id: ActiveValue::Set(self.id.clone()),
            preimage: ActiveValue::Set(preimage),
            payment_hash: ActiveValue::Set(payment_hash),
            status: ActiveValue::Set(status.to_string()),
            amt_msat: ActiveValue::Set(Some(amt_msat.try_into().unwrap())),
            origin: ActiveValue::Set(PaymentOrigin::SpontaneousOutgoing.to_string()),
            ..Default::default()
        };
        payment.insert(self.database.get_connection()).await?;

        Ok(())
    }

    pub async fn send_payment(&self, invoice: &Invoice) -> Result<(), Error> {
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

        let payment_hash = hex_utils::hex_str(&(*invoice.payment_hash()).into_inner());
        let payment_secret = Some(hex_utils::hex_str(&(*invoice.payment_secret()).0));
        let amt_msat: Option<i64> = invoice
            .amount_milli_satoshis()
            .map(|amt| amt.try_into().unwrap());

        let payment = entity::payment::ActiveModel {
            node_id: ActiveValue::Set(self.id.clone()),
            created_by_node_id: ActiveValue::Set(self.id.clone()),
            payment_hash: ActiveValue::Set(payment_hash),
            secret: ActiveValue::Set(payment_secret),
            status: ActiveValue::Set(status.to_string()),
            amt_msat: ActiveValue::Set(amt_msat),
            origin: ActiveValue::Set(PaymentOrigin::InvoiceOutgoing.to_string()),
            invoice: ActiveValue::Set(Some(invoice.to_string())),
            ..Default::default()
        };

        payment.insert(self.database.get_connection()).await?;

        Ok(())
    }

    pub async fn get_phantom_invoice(
        &self,
        amt_msat: u64,
        description: String,
        phantom_route_hints: Vec<PhantomRouteHints>,
    ) -> Result<Invoice, Error> {
        let currency = match self.config.network {
            Network::Bitcoin => Currency::Bitcoin,
            Network::Testnet => Currency::BitcoinTestnet,
            Network::Regtest => Currency::Regtest,
            Network::Signet => Currency::Signet,
        };

        let invoice = utils::create_phantom_invoice::<InMemorySigner, Arc<PhantomKeysManager>>(
            Some(amt_msat),
            None,
            description.clone(),
            3600, // FIXME invoice_expiry_delta_secs
            phantom_route_hints,
            self.keys_manager.clone(),
            currency,
        )?;

        let payment_hash = hex_utils::hex_str(&(*invoice.payment_hash()).into_inner());
        let payment_secret = Some(hex_utils::hex_str(&(*invoice.payment_secret()).0));

        let payment = entity::payment::ActiveModel {
            node_id: ActiveValue::Set(self.get_phantom_node_pubkey()),
            created_by_node_id: ActiveValue::Set(self.id.clone()),
            payment_hash: ActiveValue::Set(payment_hash),
            secret: ActiveValue::Set(payment_secret),
            status: ActiveValue::Set(HTLCStatus::Pending.to_string()),
            amt_msat: ActiveValue::Set(Some(amt_msat.try_into().unwrap())),
            origin: ActiveValue::Set(PaymentOrigin::InvoiceIncoming.to_string()),
            invoice: ActiveValue::Set(Some(invoice.to_string())),
            label: ActiveValue::Set(Some(description)),
            ..Default::default()
        };

        payment
            .insert(self.database.get_connection())
            .await
            .unwrap();

        Ok(invoice)
    }

    pub async fn get_invoice(&self, amt_msat: u64, description: String) -> Result<Invoice, Error> {
        let currency = match self.config.network {
            Network::Bitcoin => Currency::Bitcoin,
            Network::Testnet => Currency::BitcoinTestnet,
            Network::Regtest => Currency::Regtest,
            Network::Signet => Currency::Signet,
        };

        let invoice = utils::create_invoice_from_channelmanager(
            &self.channel_manager,
            self.keys_manager.clone(),
            currency,
            Some(amt_msat),
            description.clone(),
            3600, // FIXME invoice_expiry_delta_secs
        )?;

        let payment_hash = hex_utils::hex_str(&(*invoice.payment_hash()).into_inner());
        let payment_secret = Some(hex_utils::hex_str(&(*invoice.payment_secret()).0));

        let payment = entity::payment::ActiveModel {
            node_id: ActiveValue::Set(self.id.clone()),
            created_by_node_id: ActiveValue::Set(self.id.clone()),
            payment_hash: ActiveValue::Set(payment_hash),
            secret: ActiveValue::Set(payment_secret),
            status: ActiveValue::Set(HTLCStatus::Pending.to_string()),
            amt_msat: ActiveValue::Set(Some(amt_msat.try_into().unwrap())),
            origin: ActiveValue::Set(PaymentOrigin::InvoiceIncoming.to_string()),
            invoice: ActiveValue::Set(Some(invoice.to_string())),
            label: ActiveValue::Set(Some(description)),
            ..Default::default()
        };

        payment
            .insert(self.database.get_connection())
            .await
            .unwrap();

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

                channel.alias = self.get_alias_for_channel_counterparty(&chan_info);

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
    ) -> Option<String> {
        let db_alias = match self
            .database
            .find_peer_sync(&self.id, &channel_details.counterparty.node_id.to_string())
        {
            Ok(Some(peer)) => peer.alias,
            _ => None,
        };
        match db_alias {
            Some(db_alias) => Some(db_alias),
            None => {
                let node_id = NodeId::from_pubkey(&channel_details.counterparty.node_id);
                let alias = self
                    .p2p
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

                alias.map(|node_alias| node_alias.to_string())
            }
        }
    }

    pub async fn list_payments(
        &self,
        pagination: PaginationRequest,
        filter: PaymentsFilter,
    ) -> Result<(Vec<entity::payment::Model>, PaginationResponse), Error> {
        self.database
            .list_payments(self.id.clone(), pagination, filter)
            .await
    }

    pub async fn list_phantom_payments(
        &self,
        pagination: PaginationRequest,
        filter: PaymentsFilter,
    ) -> Result<(Vec<entity::payment::Model>, PaginationResponse), Error> {
        self.database
            .list_payments(self.get_phantom_node_pubkey(), pagination, filter)
            .await
    }

    pub fn close_channel(&self, channel_id: [u8; 32], force: bool) -> Result<(), Error> {
        let cp_id = self.get_channel_counterparty(&channel_id);
        if force {
            Ok(self
                .channel_manager
                .force_close_broadcasting_latest_txn(&channel_id, &cp_id)?)
        } else {
            Ok(self.channel_manager.close_channel(&channel_id, &cp_id)?)
        }
    }

    fn get_channel_counterparty(&self, channel_id: &[u8; 32]) -> PublicKey {
        let chans = self.channel_manager.list_channels();
        let chan = chans.iter().find(|c| *channel_id == c.channel_id).unwrap();
        chan.counterparty.node_id
    }

    pub fn node_info(&self) -> Result<NodeInfo, Error> {
        let chans = self.channel_manager.list_channels();
        let local_balance_msat = chans.iter().map(|c| c.balance_msat).sum::<u64>();

        Ok(NodeInfo {
            version: version::get_version(),
            node_pubkey: self.get_pubkey(),
            num_channels: chans.len() as u32,
            num_usable_channels: chans.iter().filter(|c| c.is_usable).count() as u32,
            num_peers: self.peer_manager.get_peer_node_ids().len() as u32,
            local_balance_msat,
        })
    }

    pub fn get_pubkey(&self) -> String {
        self.channel_manager.get_our_node_id().to_string()
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

    pub fn verify_message(
        &self,
        message: String,
        signature: String,
    ) -> Result<(bool, String), Error> {
        let pubkey = self.channel_manager.get_our_node_id();

        let valid =
            lightning::util::message_signing::verify(message.as_bytes(), &signature, &pubkey);

        Ok((valid, pubkey.to_string()))
    }

    pub fn list_unspent(&self) -> Result<Vec<Utxo>, Error> {
        let wallet = self.wallet.lock().unwrap();
        let local_utxos = wallet.list_unspent()?;
        let utxos = local_utxos
            .into_iter()
            .map(|u| Utxo {
                amount_sat: u.txout.value,
                spk: u.txout.script_pubkey.to_hex(),
                txid: u.outpoint.txid.to_hex(),
                output_index: u.outpoint.vout,
            })
            .collect();

        Ok(utxos)
    }

    pub async fn delete_payment(&self, payment_hash: String) -> Result<(), Error> {
        self.database
            .delete_payment(self.id.clone(), payment_hash)
            .await
    }

    pub async fn label_payment(&self, label: String, payment_hash: String) -> Result<(), Error> {
        self.database
            .label_payment(self.id.clone(), payment_hash, label)
            .await
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
            NodeRequest::GetPhantomRouteHints {} => {
                let hints = self.channel_manager.get_phantom_route_hints();
                Ok(NodeResponse::GetPhantomRouteHints {
                    phantom_route_hints_hex: hex_utils::hex_str(&hints.encode()),
                })
            }
            NodeRequest::GetBalance {} => {
                // TODO: split confirmed vs uncofirmed chain balance
                //       we currently only have 'unconfirmed' utxos from transactions we broadcast
                //       we never hear about transactions that enter the mempool
                let onchain_balance_sats = {
                    let wallet = self.wallet.lock().unwrap();
                    let balance = wallet.get_balance().map_err(Error::Bdk)?;
                    balance.get_total()
                };

                let channels = self.channel_manager.list_channels();

                let mut channel_balance_msats = 0;
                let mut channel_outbound_capacity_msats = 0;
                let mut channel_inbound_capacity_msats = 0;
                let mut usable_channel_outbound_capacity_msats = 0;
                let mut usable_channel_inbound_capacity_msats = 0;

                for channel in channels {
                    channel_balance_msats += channel.balance_msat;
                    channel_outbound_capacity_msats += channel.outbound_capacity_msat;
                    channel_inbound_capacity_msats += channel.inbound_capacity_msat;

                    if channel.is_usable {
                        usable_channel_outbound_capacity_msats += channel.outbound_capacity_msat;
                        usable_channel_inbound_capacity_msats += channel.inbound_capacity_msat;
                    }
                }

                Ok(NodeResponse::GetBalance {
                    onchain_balance_sats,
                    channel_balance_msats,
                    channel_outbound_capacity_msats,
                    channel_inbound_capacity_msats,
                    usable_channel_outbound_capacity_msats,
                    usable_channel_inbound_capacity_msats,
                })
            }
            NodeRequest::OpenChannels { requests } => self
                .open_channels(requests.clone())
                .await
                .map(|responses| NodeResponse::OpenChannels {
                    requests,
                    results: responses
                        .into_iter()
                        .map(|(_request, result)| match result {
                            Ok(channel_id) => OpenChannelResult {
                                error: false,
                                error_message: None,
                                channel_id: Some(hex_utils::hex_str(&channel_id)),
                            },
                            Err(e) => OpenChannelResult {
                                error: true,
                                error_message: Some(format!("{:?}", e)),
                                channel_id: None,
                            },
                        })
                        .collect::<Vec<_>>(),
                })
                .map_err(|e| NodeRequestError::Sensei(e.to_string())),
            NodeRequest::SendPayment { invoice } => {
                let invoice = self.get_invoice_from_str(&invoice)?;
                self.send_payment(&invoice).await?;
                Ok(NodeResponse::SendPayment {})
            }
            NodeRequest::DecodeInvoice { invoice } => {
                let invoice = self.get_invoice_from_str(&invoice)?;
                Ok(NodeResponse::DecodeInvoice {
                    invoice: invoice.into(),
                })
            }
            NodeRequest::Keysend {
                dest_pubkey,
                amt_msat,
            } => match hex_utils::to_compressed_pubkey(&dest_pubkey) {
                Some(pubkey) => {
                    self.keysend(&*self.invoice_payer, pubkey, amt_msat, &*self.keys_manager)
                        .await?;
                    Ok(NodeResponse::Keysend {})
                }
                None => Err(NodeRequestError::Sensei("invalid dest_pubkey".into())),
            },
            NodeRequest::GetInvoice {
                amt_msat,
                description,
            } => {
                let invoice = self.get_invoice(amt_msat, description).await?;
                let invoice_str = format!("{}", invoice);
                Ok(NodeResponse::GetInvoice {
                    invoice: invoice_str,
                })
            }
            NodeRequest::GetPhantomInvoice {
                amt_msat,
                description,
                phantom_route_hints_hex,
            } => {
                let phantom_route_hints = phantom_route_hints_hex
                    .into_iter()
                    .map(|hints_hex| {
                        let mut cursor = Cursor::new(hex_utils::to_vec(&hints_hex).unwrap());
                        PhantomRouteHints::read(&mut cursor).unwrap()
                    })
                    .collect::<Vec<PhantomRouteHints>>();

                let invoice = self
                    .get_phantom_invoice(amt_msat, description, phantom_route_hints)
                    .await?;
                let invoice_str = format!("{}", invoice);
                Ok(NodeResponse::GetPhantomInvoice {
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
                let (pubkey, addr) = parse_peer_info(node_connection_string).await?;

                let _res = self
                    .p2p
                    .peer_connector
                    .connect_peer_if_necessary(&self.id, pubkey, addr, self.peer_manager.clone())
                    .await;

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
                let (payments, pagination) = self.list_payments(pagination, filter).await?;
                Ok(NodeResponse::ListPayments {
                    payments,
                    pagination,
                })
            }
            NodeRequest::ListPhantomPayments { pagination, filter } => {
                let (payments, pagination) = self.list_phantom_payments(pagination, filter).await?;
                Ok(NodeResponse::ListPhantomPayments {
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
            NodeRequest::VerifyMessage { message, signature } => {
                let (valid, pubkey) = self.verify_message(message, signature)?;
                Ok(NodeResponse::VerifyMessage { valid, pubkey })
            }
            NodeRequest::ListUnspent {} => {
                let utxos = self.list_unspent()?;
                Ok(NodeResponse::ListUnspent { utxos })
            }
            NodeRequest::NetworkGraphInfo {} => {
                let graph = self.p2p.network_graph.read_only();
                let channels = graph.channels();

                let mut num_known_edge_policies: u64 = 0;
                for (_key, channel_info) in channels.iter() {
                    if channel_info.one_to_two.is_some() {
                        num_known_edge_policies += 1;
                    }
                    if channel_info.two_to_one.is_some() {
                        num_known_edge_policies += 1;
                    }
                }

                Ok(NodeResponse::NetworkGraphInfo {
                    num_channels: graph.channels().len() as u64,
                    num_nodes: graph.nodes().len() as u64,
                    num_known_edge_policies,
                })
            }
            NodeRequest::ListKnownPeers { pagination } => {
                let (peers, pagination) = self.database.list_peers(&self.id, pagination).await?;
                Ok(NodeResponse::ListKnownPeers { peers, pagination })
            }
            NodeRequest::AddKnownPeer {
                pubkey,
                label,
                zero_conf,
            } => {
                let public_key = hex_utils::to_compressed_pubkey(&pubkey);

                if public_key.is_none() {
                    return Err(NodeRequestError::Sensei(String::from("invalid pubkey")));
                }

                let node_id = NodeId::from_pubkey(&public_key.unwrap());
                let alias = self
                    .p2p
                    .peer_connector
                    .node_info_lookup
                    .get_alias(node_id)?;

                let peer = match self.database.find_peer(&self.id, &pubkey).await? {
                    Some(peer) => {
                        let mut peer: entity::peer::ActiveModel = peer.into();
                        peer.label = ActiveValue::Set(Some(label));
                        peer.zero_conf = ActiveValue::Set(zero_conf);
                        if alias.is_some() {
                            peer.alias = ActiveValue::Set(alias);
                        }
                        peer.update(self.database.get_connection())
                    }
                    None => {
                        let peer = entity::peer::ActiveModel {
                            node_id: ActiveValue::Set(self.id.clone()),
                            pubkey: ActiveValue::Set(pubkey),
                            label: ActiveValue::Set(Some(label)),
                            zero_conf: ActiveValue::Set(zero_conf),
                            alias: ActiveValue::Set(alias),
                            ..Default::default()
                        };
                        peer.insert(self.database.get_connection())
                    }
                };

                let _res = peer.await.map_err(Error::Db)?;

                Ok(NodeResponse::AddKnownPeer {})
            }
            NodeRequest::RemoveKnownPeer { pubkey } => {
                self.database.delete_peer(&self.id, &pubkey).await?;
                Ok(NodeResponse::RemoveKnownPeer {})
            }
        }
    }
}
