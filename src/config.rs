use std::{fs, io};

use bdk::blockchain::ElectrumBlockchainConfig;
use bitcoin::Network;
use serde::{Deserialize, Serialize};

pub const ELECTRUM_MAINNET_URL: &str = "ssl://blockstream.info:700";

pub const DEFAULT_SOCKS5_PROXY: Option<String> = None;
pub const DEFAULT_RETRY_ATTEMPTS: u8 = 3;
pub const DEFAULT_REQUEST_TIMEOUT_SECONDS: Option<u8> = Some(10);
pub const DEFAULT_STOP_GAP: usize = 20;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LightningNodeBackendConfig {
    #[serde(rename = "electrum")]
    Electrum(ElectrumBlockchainConfig),
}

impl Default for LightningNodeBackendConfig {
    fn default() -> Self {
        LightningNodeBackendConfig::Electrum(ElectrumBlockchainConfig {
            url: ELECTRUM_MAINNET_URL.into(),
            socks5: DEFAULT_SOCKS5_PROXY,
            retry: DEFAULT_RETRY_ATTEMPTS,
            timeout: DEFAULT_REQUEST_TIMEOUT_SECONDS,
            stop_gap: DEFAULT_STOP_GAP,
        })
    }
}
impl LightningNodeBackendConfig {
    pub fn clone_with_new_url(&self, new_url: String) -> Self {
        match self {
            LightningNodeBackendConfig::Electrum(electrum_config) => {
                LightningNodeBackendConfig::Electrum(ElectrumBlockchainConfig {
                    url: new_url,
                    socks5: electrum_config.socks5.clone(),
                    retry: electrum_config.retry,
                    timeout: electrum_config.timeout,
                    stop_gap: electrum_config.stop_gap,
                })
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SenseiConfig {
    #[serde(skip)]
    pub path: String,
    pub backend: LightningNodeBackendConfig,
    pub network: Network,
    pub api_port: u16,
}

impl Default for SenseiConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| ".".into());
        let path = format!("{}/.sensei/config.json", home_dir.to_str().unwrap());
        Self {
            path,
            backend: LightningNodeBackendConfig::default(),
            network: Network::Bitcoin,
            api_port: 5401,
        }
    }
}

impl SenseiConfig {
    pub fn from_file(path: String, merge_with: Option<SenseiConfig>) -> Self {
        let mut merge_config = merge_with.unwrap_or_default();
        merge_config.path = path.clone();

        match fs::read_to_string(path.clone()) {
            Ok(config_str) => {
                let config: SenseiConfig =
                    serde_json::from_str(&config_str).expect("failed to parse configuration file");
                // merge all of `config` properties into `merge_config`
                // return `merge_config`
                merge_config.set_backend(config.backend);
                merge_config
            }
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => {
                    fs::write(
                        path,
                        serde_json::to_string(&merge_config)
                            .expect("failed to serialize default config"),
                    )
                    .expect("failed to write default config");
                    // write merge_config to path
                    merge_config
                }
                _ => {
                    panic!("failed to read configuration file");
                }
            },
        }
    }

    pub fn set_network(&mut self, network: Network) {
        self.network = network;
    }

    pub fn set_backend(&mut self, backend: LightningNodeBackendConfig) {
        self.backend = backend;
    }

    pub fn save(&mut self) {
        fs::write(
            self.path.clone(),
            serde_json::to_string(&self).expect("failed to serialize config"),
        )
        .expect("failed to write config");
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LightningNodeConfig {
    pub backend: LightningNodeBackendConfig,
    pub data_dir: String,
    pub ldk_peer_listening_port: u16,
    pub ldk_announced_listen_addr: Vec<String>,
    pub ldk_announced_node_name: Option<String>,
    pub network: Network,
    pub passphrase: String,
    pub external_router: bool,
}

impl Default for LightningNodeConfig {
    fn default() -> Self {
        LightningNodeConfig {
            backend: LightningNodeBackendConfig::default(),
            data_dir: ".".into(),
            ldk_peer_listening_port: 9735,
            ldk_announced_listen_addr: vec![],
            ldk_announced_node_name: None,
            network: Network::Bitcoin,
            passphrase: "satoshi".into(),
            external_router: true,
        }
    }
}

impl LightningNodeConfig {
    pub fn data_dir(&self) -> String {
        format!("{}/data", self.data_dir)
    }
    pub fn node_database_path(&self) -> String {
        format!("{}/node.db", self.data_dir())
    }
    pub fn admin_macaroon_path(&self) -> String {
        format!("{}/admin.macaroon", self.data_dir())
    }
    pub fn seed_path(&self) -> String {
        format!("{}/seed", self.data_dir())
    }
    pub fn channel_manager_path(&self) -> String {
        format!("{}/manager", self.data_dir())
    }
    pub fn network_graph_path(&self) -> String {
        format!("{}/network_graph", self.data_dir())
    }
    pub fn scorer_path(&self) -> String {
        format!("{}/scorer", self.data_dir())
    }
    pub fn channel_peer_data_path(&self) -> String {
        format!("{}/channel_peer_data", self.data_dir())
    }
}
