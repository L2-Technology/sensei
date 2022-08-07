use lightning::chain::chaininterface::{ConfirmationTarget, FeeEstimator};
use tokio::runtime::Handle;

use crate::p2p::router::RemoteSenseiInfo;

pub struct RemoteFeeEstimator {
    remote_sensei: RemoteSenseiInfo,
    tokio_handle: Handle,
}

impl RemoteFeeEstimator {
    pub fn new(host: String, token: String, tokio_handle: Handle) -> Self {
        Self {
            remote_sensei: RemoteSenseiInfo { host, token },
            tokio_handle,
        }
    }

    fn fee_rate_normal_path(&self) -> String {
        format!("{}/v1/ldk/chain/fee-rate-normal", self.remote_sensei.host)
    }

    fn fee_rate_background_path(&self) -> String {
        format!(
            "{}/v1/ldk/chain/fee-rate-background",
            self.remote_sensei.host
        )
    }

    fn fee_rate_high_priority_path(&self) -> String {
        format!(
            "{}/v1/ldk/chain/fee-rate-high-priority",
            self.remote_sensei.host
        )
    }

    pub async fn get_fee_rate_normal(&self) -> u32 {
        let client = reqwest::Client::new();
        match client
            .get(self.fee_rate_normal_path())
            .header("token", self.remote_sensei.token.clone())
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(fee_rate_string) => fee_rate_string.parse().unwrap_or(2000),
                Err(_) => 2000,
            },
            Err(_) => 2000,
        }
    }

    pub async fn get_fee_rate_background(&self) -> u32 {
        let client = reqwest::Client::new();
        match client
            .get(self.fee_rate_background_path())
            .header("token", self.remote_sensei.token.clone())
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(fee_rate_string) => fee_rate_string.parse().unwrap_or(253),
                Err(_) => 253,
            },
            Err(_) => 253,
        }
    }

    pub async fn get_fee_rate_high_priority(&self) -> u32 {
        let client = reqwest::Client::new();
        match client
            .get(self.fee_rate_high_priority_path())
            .header("token", self.remote_sensei.token.clone())
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(fee_rate_string) => fee_rate_string.parse().unwrap_or(5000),
                Err(_) => 5000,
            },
            Err(_) => 5000,
        }
    }
}

impl FeeEstimator for RemoteFeeEstimator {
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        tokio::task::block_in_place(move || {
            self.tokio_handle.clone().block_on(async move {
                match confirmation_target {
                    ConfirmationTarget::Background => self.get_fee_rate_background().await,
                    ConfirmationTarget::Normal => self.get_fee_rate_normal().await,
                    ConfirmationTarget::HighPriority => self.get_fee_rate_high_priority().await,
                }
            })
        })
    }
}
