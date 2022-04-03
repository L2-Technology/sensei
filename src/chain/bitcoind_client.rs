use base64;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::consensus::encode;
use bitcoin::hash_types::{BlockHash, Txid};
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use lightning_block_sync::http::HttpEndpoint;
use lightning_block_sync::rpc::RpcClient;
use lightning_block_sync::{AsyncBlockSourceResult, BlockHeaderData, BlockSource};
use serde_json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use bitcoin::hashes::hex::FromHex;
use lightning_block_sync::http::JsonResponse;
use std::convert::TryInto;
pub struct FeeResponse {
    pub feerate_sat_per_kw: Option<u32>,
    pub errored: bool,
}

impl TryInto<FeeResponse> for JsonResponse {
    type Error = std::io::Error;
    fn try_into(self) -> std::io::Result<FeeResponse> {
        let errored = !self.0["errors"].is_null();
        Ok(FeeResponse {
            errored,
            feerate_sat_per_kw: match self.0["feerate"].as_f64() {
                // Bitcoin Core gives us a feerate in BTC/KvB, which we need to convert to
                // satoshis/KW. Thus, we first multiply by 10^8 to get satoshis, then divide by 4
                // to convert virtual-bytes into weight units.
                Some(feerate_btc_per_kvbyte) => {
                    Some((feerate_btc_per_kvbyte * 100_000_000.0 / 4.0).round() as u32)
                }
                None => None,
            },
        })
    }
}

pub struct BlockchainInfo {
    pub latest_height: usize,
    pub latest_blockhash: BlockHash,
    pub chain: String,
}

impl TryInto<BlockchainInfo> for JsonResponse {
    type Error = std::io::Error;
    fn try_into(self) -> std::io::Result<BlockchainInfo> {
        Ok(BlockchainInfo {
            latest_height: self.0["blocks"].as_u64().unwrap() as usize,
            latest_blockhash: BlockHash::from_hex(self.0["bestblockhash"].as_str().unwrap())
                .unwrap(),
            chain: self.0["chain"].as_str().unwrap().to_string(),
        })
    }
}
pub struct BitcoindClient {
    bitcoind_rpc_client: Arc<Mutex<RpcClient>>,
    host: String,
    port: u16,
    rpc_user: String,
    rpc_password: String,
    fees: Arc<HashMap<Target, AtomicU32>>,
    handle: tokio::runtime::Handle,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Target {
    Background,
    Normal,
    HighPriority,
}

impl BlockSource for &BitcoindClient {
    fn get_header<'a>(
        &'a mut self,
        header_hash: &'a BlockHash,
        height_hint: Option<u32>,
    ) -> AsyncBlockSourceResult<'a, BlockHeaderData> {
        Box::pin(async move {
            let mut rpc = self.bitcoind_rpc_client.lock().await;
            rpc.get_header(header_hash, height_hint).await
        })
    }

    fn get_block<'a>(
        &'a mut self,
        header_hash: &'a BlockHash,
    ) -> AsyncBlockSourceResult<'a, Block> {
        Box::pin(async move {
            let mut rpc = self.bitcoind_rpc_client.lock().await;
            rpc.get_block(header_hash).await
        })
    }

    fn get_best_block<'a>(&'a mut self) -> AsyncBlockSourceResult<(BlockHash, Option<u32>)> {
        Box::pin(async move {
            let mut rpc = self.bitcoind_rpc_client.lock().await;
            rpc.get_best_block().await
        })
    }
}

/// The minimum feerate we are allowed to send, as specify by LDK.
const MIN_FEERATE: u32 = 253;

impl BitcoindClient {
    pub async fn new(
        host: String,
        port: u16,
        rpc_user: String,
        rpc_password: String,
        handle: tokio::runtime::Handle,
    ) -> std::io::Result<Self> {
        let http_endpoint = HttpEndpoint::for_host(host.clone()).with_port(port);
        let rpc_credentials =
            base64::encode(format!("{}:{}", rpc_user.clone(), rpc_password.clone()));
        let mut bitcoind_rpc_client = RpcClient::new(&rpc_credentials, http_endpoint)?;
        let _dummy = bitcoind_rpc_client
            .call_method::<BlockchainInfo>("getblockchaininfo", &vec![])
            .await
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::PermissionDenied,
				"Failed to make initial call to bitcoind - please check your RPC user/password and access settings")
            })?;
        let mut fees: HashMap<Target, AtomicU32> = HashMap::new();
        fees.insert(Target::Background, AtomicU32::new(MIN_FEERATE));
        fees.insert(Target::Normal, AtomicU32::new(2000));
        fees.insert(Target::HighPriority, AtomicU32::new(5000));
        let client = Self {
            bitcoind_rpc_client: Arc::new(Mutex::new(bitcoind_rpc_client)),
            host,
            port,
            rpc_user,
            rpc_password,
            fees: Arc::new(fees),
            handle: handle.clone(),
        };
        BitcoindClient::poll_for_fee_estimates(
            client.fees.clone(),
            client.bitcoind_rpc_client.clone(),
            handle,
        );
        Ok(client)
    }

    fn poll_for_fee_estimates(
        fees: Arc<HashMap<Target, AtomicU32>>,
        rpc_client: Arc<Mutex<RpcClient>>,
        handle: tokio::runtime::Handle,
    ) {
        handle.spawn(async move {
            loop {
                let background_estimate = {
                    let mut rpc = rpc_client.lock().await;
                    let background_conf_target = serde_json::json!(144);
                    let background_estimate_mode = serde_json::json!("ECONOMICAL");
                    let resp = rpc
                        .call_method::<FeeResponse>(
                            "estimatesmartfee",
                            &vec![background_conf_target, background_estimate_mode],
                        )
                        .await
                        .unwrap();
                    match resp.feerate_sat_per_kw {
                        Some(feerate) => std::cmp::max(feerate, MIN_FEERATE),
                        None => MIN_FEERATE,
                    }
                };

                let normal_estimate = {
                    let mut rpc = rpc_client.lock().await;
                    let normal_conf_target = serde_json::json!(18);
                    let normal_estimate_mode = serde_json::json!("ECONOMICAL");
                    let resp = rpc
                        .call_method::<FeeResponse>(
                            "estimatesmartfee",
                            &vec![normal_conf_target, normal_estimate_mode],
                        )
                        .await
                        .unwrap();
                    match resp.feerate_sat_per_kw {
                        Some(feerate) => std::cmp::max(feerate, MIN_FEERATE),
                        None => 2000,
                    }
                };

                let high_prio_estimate = {
                    let mut rpc = rpc_client.lock().await;
                    let high_prio_conf_target = serde_json::json!(6);
                    let high_prio_estimate_mode = serde_json::json!("CONSERVATIVE");
                    let resp = rpc
                        .call_method::<FeeResponse>(
                            "estimatesmartfee",
                            &vec![high_prio_conf_target, high_prio_estimate_mode],
                        )
                        .await
                        .unwrap();

                    match resp.feerate_sat_per_kw {
                        Some(feerate) => std::cmp::max(feerate, MIN_FEERATE),
                        None => 5000,
                    }
                };

                fees.get(&Target::Background)
                    .unwrap()
                    .store(background_estimate, Ordering::Release);
                fees.get(&Target::Normal)
                    .unwrap()
                    .store(normal_estimate, Ordering::Release);
                fees.get(&Target::HighPriority)
                    .unwrap()
                    .store(high_prio_estimate, Ordering::Release);
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    pub fn get_new_rpc_client(&self) -> std::io::Result<RpcClient> {
        let http_endpoint = HttpEndpoint::for_host(self.host.clone()).with_port(self.port);
        let rpc_credentials = base64::encode(format!(
            "{}:{}",
            self.rpc_user.clone(),
            self.rpc_password.clone()
        ));
        RpcClient::new(&rpc_credentials, http_endpoint)
    }

    pub async fn send_raw_transaction(&self, raw_tx: String) {
        let mut rpc = self.bitcoind_rpc_client.lock().await;

        let raw_tx_json = serde_json::json!(raw_tx);
        rpc.call_method::<Txid>("sendrawtransaction", &[raw_tx_json])
            .await
            .unwrap();
    }

    pub async fn get_blockchain_info(&self) -> BlockchainInfo {
        let mut rpc = self.bitcoind_rpc_client.lock().await;
        rpc.call_method::<BlockchainInfo>("getblockchaininfo", &vec![])
            .await
            .unwrap()
    }
}

impl FeeEstimator for BitcoindClient {
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        match confirmation_target {
            ConfirmationTarget::Background => self
                .fees
                .get(&Target::Background)
                .unwrap()
                .load(Ordering::Acquire),
            ConfirmationTarget::Normal => self
                .fees
                .get(&Target::Normal)
                .unwrap()
                .load(Ordering::Acquire),
            ConfirmationTarget::HighPriority => self
                .fees
                .get(&Target::HighPriority)
                .unwrap()
                .load(Ordering::Acquire),
        }
    }
}

impl BroadcasterInterface for BitcoindClient {
    fn broadcast_transaction(&self, tx: &Transaction) {
        let bitcoind_rpc_client = self.bitcoind_rpc_client.clone();
        let tx_serialized = serde_json::json!(encode::serialize_hex(tx));
        self.handle.spawn(async move {
            let mut rpc = bitcoind_rpc_client.lock().await;
            // This may error due to RL calling `broadcast_transaction` with the same transaction
            // multiple times, but the error is safe to ignore.
            match rpc
                .call_method::<Txid>("sendrawtransaction", &vec![tx_serialized])
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    let err_str = e.get_ref().unwrap().to_string();
                    if !err_str.contains("Transaction already in block chain")
                        && !err_str.contains("Inputs missing or spent")
                        && !err_str.contains("bad-txns-inputs-missingorspent")
                        && !err_str.contains("non-BIP68-final")
                        && !err_str.contains("insufficient fee, rejecting replacement ")
                    {
                        panic!("{}", e);
                    }
                }
            }
        });
    }
}
