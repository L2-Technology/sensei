// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::{fs, io};

use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SenseiConfig {
    #[serde(skip)]
    pub path: String,
    pub bitcoind_rpc_host: String,
    pub bitcoind_rpc_port: u16,
    pub bitcoind_rpc_username: String,
    pub bitcoind_rpc_password: String,
    pub network: Network,
    pub api_port: u16,
}

impl Default for SenseiConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| ".".into());
        let path = format!("{}/.sensei/config.json", home_dir.to_str().unwrap());
        Self {
            path,
            bitcoind_rpc_host: String::from("127.0.0.1"),
            bitcoind_rpc_port: 8133,
            bitcoind_rpc_username: String::from("bitcoin"),
            bitcoind_rpc_password: String::from("bitcoin"),
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
                merge_config.bitcoind_rpc_host = config.bitcoind_rpc_host;
                merge_config.bitcoind_rpc_port = config.bitcoind_rpc_port;
                merge_config.bitcoind_rpc_username = config.bitcoind_rpc_username;
                merge_config.bitcoind_rpc_password = config.bitcoind_rpc_password;
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

    pub fn bdk_database_path(&self) -> String {
        format!("{}/bdk.db", self.data_dir())
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
