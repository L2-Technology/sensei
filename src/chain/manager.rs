use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    config::SenseiConfig,
    node::{ChainMonitor, ChannelManager},
};
use bitcoin::BlockHash;
use lightning::chain::{
    chaininterface::{BroadcasterInterface, FeeEstimator},
    BestBlock, Listen,
};
use lightning_block_sync::SpvClient;
use lightning_block_sync::{init, poll, UnboundedCache};
use lightning_block_sync::{poll::ValidatedBlockHeader, BlockSource};
use std::ops::Deref;

use super::{database::WalletDatabase, listener::SenseiChainListener};

pub struct SenseiChainManager {
    config: SenseiConfig,
    pub listener: Arc<SenseiChainListener>,
    pub block_source: Arc<dyn BlockSource + Send + Sync>,
    pub fee_estimator: Arc<dyn FeeEstimator + Send + Sync>,
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    poller_paused: Arc<AtomicBool>,
}

impl SenseiChainManager {
    pub async fn new(
        config: SenseiConfig,
        block_source: Arc<dyn BlockSource + Send + Sync>,
        fee_estimator: Arc<dyn FeeEstimator + Send + Sync>,
        broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    ) -> Result<Self, crate::error::Error> {
        let listener = Arc::new(SenseiChainListener::new());
        let block_source_poller = block_source.clone();
        let listener_poller = listener.clone();
        let poller_paused = Arc::new(AtomicBool::new(false));
        let poller_paused_poller = poller_paused.clone();
        tokio::spawn(async move {
            let mut cache = UnboundedCache::new();
            let chain_tip = init::validate_best_block_header(block_source_poller.clone())
                .await
                .unwrap();
            let chain_poller = poll::ChainPoller::new(block_source_poller, config.network);
            let mut spv_client =
                SpvClient::new(chain_tip, chain_poller, &mut cache, listener_poller);
            loop {
                if !poller_paused_poller.load(Ordering::Relaxed) {
                    let _tip = spv_client.poll_best_tip().await.unwrap();
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            config,
            listener,
            poller_paused,
            block_source,
            fee_estimator,
            broadcaster,
        })
    }

    pub async fn synchronize_to_tip(
        &self,
        chain_listeners: Vec<(BlockHash, &(dyn Listen + Send + Sync))>,
    ) -> Result<ValidatedBlockHeader, crate::error::Error> {
        let chain_tip = init::synchronize_listeners(
            self.block_source.clone(),
            self.config.network,
            &mut UnboundedCache::new(),
            chain_listeners,
        )
        .await
        .unwrap();

        Ok(chain_tip)
    }

    pub async fn keep_in_sync(
        &self,
        synced_hash: BlockHash,
        channel_manager: Arc<ChannelManager>,
        chain_monitor: Arc<ChainMonitor>,
        wallet_database: WalletDatabase,
    ) -> Result<(), crate::error::Error> {
        let listeners = vec![
            (
                synced_hash,
                channel_manager.deref() as &(dyn Listen + Send + Sync),
            ),
            (
                synced_hash,
                chain_monitor.deref() as &(dyn Listen + Send + Sync),
            ),
            (synced_hash, &wallet_database as &(dyn Listen + Send + Sync)),
        ];

        self.poller_paused.store(true, Ordering::Relaxed);
        // could skip this if synced_hash === current_tip
        let _new_tip = self.synchronize_to_tip(listeners).await.unwrap();
        self.listener
            .add_listener((chain_monitor, channel_manager, wallet_database));
        self.poller_paused.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub async fn get_best_block(&self) -> Result<BestBlock, crate::error::Error> {
        let (latest_blockhash, latest_height) = self.block_source.get_best_block().await.unwrap();
        Ok(BestBlock::new(latest_blockhash, latest_height.unwrap()))
    }
}
