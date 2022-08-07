use bitcoin::Network;
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use lightning_block_sync::BlockSource;
use tokio::runtime::Handle;

use self::{
    bitcoind_client::BitcoindClient,
    remote::{
        block_source::RemoteBlockSource, broadcaster::RemoteBroadcaster,
        fee_estimator::RemoteFeeEstimator,
    },
};
use std::sync::Arc;

pub mod bitcoind_client;
pub mod broadcaster;
pub mod database;
pub mod fee_estimator;
pub mod listener;
pub mod manager;
pub mod remote;

pub enum AnyBlockSource {
    Local(Arc<BitcoindClient>),
    Remote(remote::block_source::RemoteBlockSource),
}

impl AnyBlockSource {
    pub fn new_remote(network: Network, host: String, token: String) -> Self {
        AnyBlockSource::Remote(RemoteBlockSource::new(network, host, token))
    }
}

impl BlockSource for AnyBlockSource {
    fn get_header<'a>(
        &'a self,
        header_hash: &'a bitcoin::BlockHash,
        height_hint: Option<u32>,
    ) -> lightning_block_sync::AsyncBlockSourceResult<'a, lightning_block_sync::BlockHeaderData>
    {
        match self {
            AnyBlockSource::Local(bitcoind_client) => {
                bitcoind_client.get_header(header_hash, height_hint)
            }
            AnyBlockSource::Remote(remote) => remote.get_header(header_hash, height_hint),
        }
    }

    fn get_block<'a>(
        &'a self,
        header_hash: &'a bitcoin::BlockHash,
    ) -> lightning_block_sync::AsyncBlockSourceResult<'a, bitcoin::Block> {
        match self {
            AnyBlockSource::Local(bitcoind_client) => bitcoind_client.get_block(header_hash),
            AnyBlockSource::Remote(remote) => remote.get_block(header_hash),
        }
    }

    fn get_best_block(
        &self,
    ) -> lightning_block_sync::AsyncBlockSourceResult<(bitcoin::BlockHash, Option<u32>)> {
        match self {
            AnyBlockSource::Local(bitcoind_client) => bitcoind_client.get_best_block(),
            AnyBlockSource::Remote(remote) => remote.get_best_block(),
        }
    }
}

pub enum AnyFeeEstimator {
    Local(Arc<BitcoindClient>),
    Remote(RemoteFeeEstimator),
}

impl AnyFeeEstimator {
    pub fn new_remote(host: String, token: String, handle: Handle) -> Self {
        AnyFeeEstimator::Remote(RemoteFeeEstimator::new(host, token, handle))
    }
}

impl FeeEstimator for AnyFeeEstimator {
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        match self {
            AnyFeeEstimator::Local(bitcoind_client) => {
                bitcoind_client.get_est_sat_per_1000_weight(confirmation_target)
            }
            AnyFeeEstimator::Remote(remote) => {
                remote.get_est_sat_per_1000_weight(confirmation_target)
            }
        }
    }
}

pub enum AnyBroadcaster {
    Local(Arc<BitcoindClient>),
    Remote(RemoteBroadcaster),
}

impl AnyBroadcaster {
    pub fn new_remote(host: String, token: String, handle: Handle) -> Self {
        AnyBroadcaster::Remote(RemoteBroadcaster::new(host, token, handle))
    }
}

impl BroadcasterInterface for AnyBroadcaster {
    fn broadcast_transaction(&self, tx: &bitcoin::Transaction) {
        match self {
            AnyBroadcaster::Local(bitcoind_client) => bitcoind_client.broadcast_transaction(tx),
            AnyBroadcaster::Remote(remote) => remote.broadcast_transaction(tx),
        }
    }
}
