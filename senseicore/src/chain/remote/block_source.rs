use crate::{hex_utils, p2p::router::RemoteSenseiInfo};
use bitcoin::{
    consensus::deserialize,
    hashes::hex::{FromHex, ToHex},
    util::uint::Uint256,
    Block, BlockHash, BlockHeader, Network,
};
use lightning::chain::BestBlock;
use lightning_block_sync::{BlockHeaderData, BlockSource, BlockSourceError};

pub struct RemoteBlockSource {
    network: Network,
    remote_sensei: RemoteSenseiInfo,
}

impl RemoteBlockSource {
    pub fn new(network: Network, host: String, token: String) -> Self {
        Self {
            network,
            remote_sensei: RemoteSenseiInfo { host, token },
        }
    }
    fn get_header_path(&self, header_hash: String) -> String {
        format!(
            "{}/v1/ldk/chain/header/{}",
            self.remote_sensei.host, header_hash
        )
    }
    fn get_block_path(&self, header_hash: String) -> String {
        format!(
            "{}/v1/ldk/chain/block/{}",
            self.remote_sensei.host, header_hash
        )
    }
    fn get_best_block_hash_path(&self) -> String {
        format!("{}/v1/ldk/chain/best-block-hash", self.remote_sensei.host)
    }
    fn get_best_block_height_path(&self) -> String {
        format!("{}/v1/ldk/chain/best-block-height", self.remote_sensei.host)
    }

    pub async fn get_best_block_hash(&self) -> Option<BlockHash> {
        let client = reqwest::Client::new();
        match client
            .get(self.get_best_block_hash_path())
            .header("token", self.remote_sensei.token.clone())
            .send()
            .await
        {
            Ok(response) => match response.bytes().await {
                Ok(serialized_hash) => Some(deserialize(&serialized_hash).unwrap()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub async fn get_best_block_height(&self) -> Option<u32> {
        let client = reqwest::Client::new();
        match client
            .get(self.get_best_block_height_path())
            .header("token", self.remote_sensei.token.clone())
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(height_as_string) => Some(height_as_string.parse().unwrap()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub async fn get_best_block_async(&self) -> BestBlock {
        let best_hash = self.get_best_block_hash().await;
        let best_height = self.get_best_block_height().await;
        if best_hash.is_none() || best_height.is_none() {
            BestBlock::from_genesis(self.network)
        } else {
            BestBlock::new(best_hash.unwrap(), best_height.unwrap())
        }
    }
}

impl BlockSource for RemoteBlockSource {
    fn get_header<'a>(
        &'a self,
        header_hash: &'a bitcoin::BlockHash,
        _height_hint: Option<u32>,
    ) -> lightning_block_sync::AsyncBlockSourceResult<'a, lightning_block_sync::BlockHeaderData>
    {
        Box::pin(async move {
            let client = reqwest::Client::new();
            let res = client
                .get(self.get_header_path(header_hash.to_hex()))
                .header("token", self.remote_sensei.token.clone())
                .send()
                .await;

            match res {
                Ok(response) => match response.text().await {
                    Ok(header_data_string) => {
                        let header_parts: Vec<&str> = header_data_string.split(',').collect();
                        let header: BlockHeader =
                            deserialize(&Vec::<u8>::from_hex(header_parts[0]).unwrap()).unwrap();
                        let height: u32 = header_parts[1].to_string().parse().unwrap();
                        let chainwork: Uint256 =
                            deserialize(&hex_utils::to_vec(header_parts[2]).unwrap()).unwrap();

                        Ok(BlockHeaderData {
                            header,
                            height,
                            chainwork,
                        })
                    }
                    Err(e) => Err(BlockSourceError::transient(e)),
                },
                Err(e) => Err(BlockSourceError::transient(e)),
            }
        })
    }

    fn get_block<'a>(
        &'a self,
        header_hash: &'a bitcoin::BlockHash,
    ) -> lightning_block_sync::AsyncBlockSourceResult<'a, bitcoin::Block> {
        Box::pin(async move {
            let client = reqwest::Client::new();
            let res = client
                .get(self.get_block_path(header_hash.to_hex()))
                .header("token", self.remote_sensei.token.clone())
                .send()
                .await;

            match res {
                Ok(response) => match response.bytes().await {
                    Ok(serialized_block_data) => {
                        let block: Block = deserialize(&serialized_block_data).unwrap();
                        Ok(block)
                    }
                    Err(e) => Err(BlockSourceError::transient(e)),
                },
                Err(e) => Err(BlockSourceError::transient(e)),
            }
        })
    }

    fn get_best_block(
        &self,
    ) -> lightning_block_sync::AsyncBlockSourceResult<(bitcoin::BlockHash, Option<u32>)> {
        Box::pin(async move {
            let best_block = self.get_best_block_async().await;
            Ok((best_block.block_hash(), Some(best_block.height())))
        })
    }
}
