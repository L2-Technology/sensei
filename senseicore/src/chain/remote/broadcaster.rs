use crate::{hex_utils, p2p::router::RemoteSenseiInfo};
use lightning::chain::chaininterface::BroadcasterInterface;
use lightning::util::ser::Writeable;
use tokio::runtime::Handle;

pub struct RemoteBroadcaster {
    remote_sensei: RemoteSenseiInfo,
    tokio_handle: Handle,
}

impl RemoteBroadcaster {
    pub fn new(host: String, token: String, tokio_handle: Handle) -> Self {
        Self {
            remote_sensei: RemoteSenseiInfo { host, token },
            tokio_handle,
        }
    }

    fn broadcast_path(&self) -> String {
        format!("{}/v1/ldk/chain/broadcast", self.remote_sensei.host)
    }

    pub async fn broadcast_transaction_async(&self, tx: &bitcoin::Transaction) {
        let client = reqwest::Client::new();
        let _res = client
            .post(self.broadcast_path())
            .header("token", self.remote_sensei.token.clone())
            .json(&serde_json::json!({
              "tx": hex_utils::hex_str(&tx.encode())
            }))
            .send()
            .await;
    }
}

impl BroadcasterInterface for RemoteBroadcaster {
    fn broadcast_transaction(&self, tx: &bitcoin::Transaction) {
        tokio::task::block_in_place(move || {
            self.tokio_handle
                .clone()
                .block_on(async move { self.broadcast_transaction_async(tx).await })
        })
    }
}
