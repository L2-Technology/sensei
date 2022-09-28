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
use serde_json::Value;

#[derive(Clone)]
pub enum P2PConfig {
    Local,
    Remote(String, String),
    RapidGossipSync(String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SenseiConfig {
    #[serde(skip)]
    pub path: String,
    pub bitcoind_rpc_host: String,
    pub bitcoind_rpc_port: u16,
    pub bitcoind_rpc_username: String,
    pub bitcoind_rpc_password: String,
    pub network: Network,
    pub api_host: String,
    pub api_port: u16,
    pub port_range_min: u16,
    pub port_range_max: u16,
    pub database_url: String,
    pub remote_p2p_host: Option<String>,
    pub remote_p2p_token: Option<String>,
    pub remote_chain_host: Option<String>,
    pub remote_chain_token: Option<String>,
    pub gossip_peers: String,
    pub instance_name: String,
    pub http_notifier_url: Option<String>,
    pub http_notifier_token: Option<String>,
    pub region: Option<String>,
    pub poll_for_chain_updates: bool,
    pub rapid_gossip_sync_server_host: Option<String>,
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
            api_host: String::from("127.0.0.1"),
            api_port: 5401,
            port_range_min: 10000,
            port_range_max: 65535,
            database_url: String::from("sensei.db"),
            remote_p2p_host: None,
            remote_p2p_token: None,
            remote_chain_host: None,
            remote_chain_token: None,
            gossip_peers: String::from(""),
            instance_name: String::from("sensei"),
            http_notifier_url: None,
            http_notifier_token: None,
            region: None,
            poll_for_chain_updates: true,
            rapid_gossip_sync_server_host: None,
        }
    }
}

impl SenseiConfig {
    pub fn from_file(path: String, merge_with: Option<SenseiConfig>) -> Self {
        let mut merge_config = merge_with.unwrap_or_default();
        merge_config.path = path.clone();

        match fs::read_to_string(path.clone()) {
            Ok(config_str) => {
                let mut merge_config_value = serde_json::to_value(merge_config).unwrap();
                let merge_config_map = merge_config_value.as_object_mut().unwrap();
                let mut config_value: Value =
                    serde_json::from_str(&config_str).expect("failed to parse configuration file");
                let config_map = config_value
                    .as_object_mut()
                    .expect("failed to parse configuration file");

                merge_config_map.append(config_map);
                serde_json::from_value(merge_config_value).unwrap()
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

    pub fn get_p2p_config(&self) -> P2PConfig {
        if self.remote_p2p_configured() {
            P2PConfig::Remote(
                self.remote_p2p_host.as_ref().unwrap().clone(),
                self.remote_p2p_token.as_ref().unwrap().clone(),
            )
        } else if self.rapid_gossip_sync_configured() {
            P2PConfig::RapidGossipSync(self.rapid_gossip_sync_server_host.as_ref().unwrap().clone())
        } else {
            P2PConfig::Local
        }
    }

    pub fn remote_p2p_configured(&self) -> bool {
        self.remote_p2p_host.is_some() && self.remote_p2p_token.is_some()
    }

    pub fn rapid_gossip_sync_configured(&self) -> bool {
        self.rapid_gossip_sync_server_host.is_some()
    }
}
