use lightning::chain::chaininterface::{ConfirmationTarget, FeeEstimator};
use std::sync::Arc;

pub struct SenseiFeeEstimator {
    pub fee_estimator: Arc<dyn FeeEstimator + Send + Sync>,
}

impl FeeEstimator for SenseiFeeEstimator {
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        self.fee_estimator
            .get_est_sat_per_1000_weight(confirmation_target)
    }
}
