use std::{sync::Arc, time::Duration};

use crate::{
    config::SenseiConfig,
    node::{ChainMonitor, ChannelManager},
};
use bitcoin::BlockHash;
use lightning::chain::{BestBlock, Listen};
use lightning_block_sync::{poll::ValidatedBlockHeader, BlockSource};
use lightning_block_sync::SpvClient;
use lightning_block_sync::{init, poll, UnboundedCache};
use std::ops::Deref;

use super::{
    bitcoind_client::BitcoindClient, listener::SenseiChainListener,
    listener_database::ListenerDatabase,
};

pub struct SenseiChainManager {
    config: SenseiConfig,
    pub listener: Arc<SenseiChainListener>,
    pub bitcoind_client: Arc<BitcoindClient>,
}

impl SenseiChainManager {
    pub async fn new(config: SenseiConfig) -> Result<Self, crate::error::Error> {
        let listener = Arc::new(SenseiChainListener::new());

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
        
        let block_source_poller = bitcoind_client.clone();
        let listener_poller = listener.clone();
        tokio::spawn(async move {
            let derefed = &mut block_source_poller.deref();
            let mut cache = UnboundedCache::new();
            let chain_tip = init::validate_best_block_header(derefed).await.unwrap();
            let chain_poller = poll::ChainPoller::new(derefed, config.network);
            let mut spv_client =
                SpvClient::new(chain_tip, chain_poller, &mut cache, listener_poller);
            loop {
                spv_client.poll_best_tip().await.unwrap();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            config,
            listener,
            bitcoind_client,
        })
    }

    pub async fn synchronize_to_tip(
        &self,
        chain_listeners: Vec<(BlockHash, &(dyn Listen + Send + Sync))>,
    ) -> Result<ValidatedBlockHeader, crate::error::Error> {
        let chain_tip = init::synchronize_listeners(
            &mut self.bitcoind_client.deref(),
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
        channel_manager: Arc<ChannelManager>,
        chain_monitor: Arc<ChainMonitor>,
        listener_database: ListenerDatabase,
    ) -> Result<(), crate::error::Error> {
        let chain_listener = (chain_monitor, channel_manager, listener_database);
        self.listener.add_listener(chain_listener);
        Ok(())
    }

    pub async fn get_best_block(&self) -> Result<BestBlock, crate::error::Error> {
        let mut block_source = self.bitcoind_client.deref();
        let (latest_blockhash, latest_height) = block_source.get_best_block().await.unwrap();
        Ok(BestBlock::new(
            latest_blockhash,
            latest_height.unwrap(),
        ))
    }
}
