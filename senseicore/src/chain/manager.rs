use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
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
    Listen,
};
use lightning_block_sync::{init, poll, UnboundedCache};
use lightning_block_sync::{poll::ChainTip, SpvClient};
use lightning_block_sync::{poll::ValidatedBlockHeader, BlockSource};
use std::ops::Deref;
use tokio::{sync::Mutex, task::JoinHandle};

use super::{
    database::WalletDatabase, fee_estimator::SenseiFeeEstimator, listener::SenseiChainListener,
};

pub struct SenseiChainManager {
    config: SenseiConfig,
    pub listener: Arc<SenseiChainListener>,
    pub block_source: Arc<dyn BlockSource + Send + Sync>,
    pub fee_estimator: Arc<SenseiFeeEstimator>,
    pub broadcaster: Arc<dyn BroadcasterInterface + Send + Sync>,
    poller_paused: Arc<AtomicBool>,
    poller_running: Arc<AtomicBool>,
    chain_update_available: Arc<AtomicUsize>,
    poller_handle: Mutex<Option<JoinHandle<()>>>,
    pub current_tip: Arc<Mutex<ValidatedBlockHeader>>,
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
        let poller_running = Arc::new(AtomicBool::new(true));

        let chain_update_available = Arc::new(AtomicUsize::new(0));
        let chain_update_available_poller = chain_update_available.clone();

        let poller_paused_poller = poller_paused.clone();
        let poller_running_poller = poller_running.clone();

        let chain_tip = init::validate_best_block_header(block_source.clone())
            .await
            .unwrap();

        let current_tip = Arc::new(Mutex::new(chain_tip));
        let current_tip_poller = current_tip.clone();

        let poller_handle = tokio::spawn(async move {
            let mut cache = UnboundedCache::new();

            let chain_poller = poll::ChainPoller::new(block_source_poller, config.network);
            let mut spv_client =
                SpvClient::new(chain_tip, chain_poller, &mut cache, listener_poller);
            while poller_running_poller.load(Ordering::Relaxed) {
                let updates_available = chain_update_available_poller.load(Ordering::Relaxed) > 0;
                let paused = poller_paused_poller.load(Ordering::Relaxed);
                if (config.poll_for_chain_updates || updates_available) && !paused {
                    let (tip, _new_blocks) = spv_client.poll_best_tip().await.unwrap();

                    let new_tip = match tip {
                        ChainTip::Common => None,
                        ChainTip::Better(new_tip) => Some(new_tip),
                        ChainTip::Worse(new_tip) => Some(new_tip),
                    };

                    if let Some(new_tip) = new_tip {
                        let mut current_tip = current_tip_poller.lock().await;
                        *current_tip = new_tip;
                    }

                    if updates_available {
                        chain_update_available_poller.fetch_sub(1, Ordering::Relaxed);
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            config,
            listener,
            poller_paused,
            poller_running,
            chain_update_available,
            block_source,
            fee_estimator: Arc::new(SenseiFeeEstimator { fee_estimator }),
            broadcaster,
            poller_handle: Mutex::new(Some(poller_handle)),
            current_tip,
        })
    }

    pub fn chain_updated(&self) {
        self.chain_update_available.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn stop(&self) {
        self.poller_running.store(false, Ordering::Relaxed);
        let handle = self.poller_handle.lock().await.take().unwrap();
        handle.abort();
        handle.await.unwrap_or_default();
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

    pub fn pause_poller(&self) {
        self.poller_paused.store(true, Ordering::Relaxed);
    }

    pub fn resume_poller(&self) {
        self.poller_paused.store(false, Ordering::Relaxed);
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

        // could skip this if synced_hash === current_tip
        let _new_tip = self.synchronize_to_tip(listeners).await.unwrap();
        self.listener
            .add_listener((chain_monitor, channel_manager, wallet_database));

        Ok(())
    }

    pub async fn get_current_tip(&self) -> Result<ValidatedBlockHeader, crate::error::Error> {
        let current_tip = self.current_tip.lock().await;
        Ok(*current_tip)
    }
}
