use std::collections::HashMap;

use bitcoin::{secp256k1::PublicKey, Script, Txid};
use serde::Serialize;
use tokio::{sync::broadcast, task::JoinHandle};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "name", content = "payload")]
pub enum SenseiEvent {
    InstanceStarted {
        instance_name: String,
        api_host: String,
        api_port: u16,
        network: String,
        version: String,
        region: Option<String>,
    },
    InstanceStopped {
        instance_name: String,
        api_host: String,
    },
    TransactionBroadcast {
        node_id: String,
        txid: Txid,
    },
    FundingGenerationReady {
        node_id: String,
        temporary_channel_id: [u8; 32],
        channel_value_satoshis: u64,
        output_script: Script,
        user_channel_id: u64,
        counterparty_node_id: PublicKey,
    },
    ChannelClosed {
        node_id: String,
        channel_id: [u8; 32],
        user_channel_id: u64,
        reason: String,
    },
}

pub struct LogNotifier {}
impl LogNotifier {
    fn notify(&self, event: SenseiEvent) {
        println!("EVENT {:?}", serde_json::to_string(&event).unwrap());
    }
}

pub struct HttpNotifier {
    pub url: String,
    pub token: String,
}
impl HttpNotifier {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }

    pub async fn notify(&self, event: SenseiEvent) {
        let client = reqwest::Client::new();
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("event".to_string(), serde_json::to_string(&event).unwrap());
        let _res = client.post(&self.url).json(&map).send().await;
    }
}

pub enum AnyNotifier {
    Log(LogNotifier),
    Http(HttpNotifier),
}

impl AnyNotifier {
    pub fn new_log() -> Self {
        Self::Log(LogNotifier {})
    }

    pub fn new_http(url: String, token: String) -> Self {
        Self::Http(HttpNotifier::new(url, token))
    }
}

pub struct EventService {}

impl EventService {
    pub fn listen(
        handle: tokio::runtime::Handle,
        notifier: AnyNotifier,
        mut event_receiver: broadcast::Receiver<SenseiEvent>,
    ) -> JoinHandle<()> {
        handle.spawn(async move {
            loop {
                match event_receiver.recv().await {
                    Ok(event) => match &notifier {
                        AnyNotifier::Log(notifier) => notifier.notify(event),
                        AnyNotifier::Http(notifier) => notifier.notify(event).await,
                    },
                    Err(err) => match err {
                        broadcast::error::RecvError::Closed => {
                            break;
                        }
                        broadcast::error::RecvError::Lagged(_skipped) => {}
                    },
                }
            }
        })
    }
}
