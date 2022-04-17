use bdk::bitcoin::{Address, Script, Transaction};
use bdk::blockchain::{noop_progress, Blockchain};
use bdk::database::BatchDatabase;
use bdk::wallet::{AddressIndex, Wallet};
use bdk::SignOptions;

use crate::error::Error;
use lightning::chain::chaininterface::BroadcasterInterface;
use lightning::chain::chaininterface::{ConfirmationTarget, FeeEstimator};
use std::sync::{Mutex, MutexGuard};

/// Lightning Wallet
///
/// A wrapper around a bdk::Wallet to fulfill many of the requirements
/// needed to use lightning with LDK.
pub struct LightningWallet<B, D> {
    inner: Mutex<Wallet<B, D>>,
}

impl<B, D> LightningWallet<B, D>
where
    B: Blockchain,
    D: BatchDatabase,
{
    /// create a new lightning wallet from your bdk wallet
    pub fn new(wallet: Wallet<B, D>) -> Self {
        LightningWallet {
            inner: Mutex::new(wallet),
        }
    }

    #[allow(dead_code)]
    pub fn get_unused_address(&self) -> Result<Address, Error> {
        let wallet = self.inner.lock().unwrap();
        let address_info = wallet.get_address(AddressIndex::LastUnused)?;
        Ok(address_info.address)
    }

    #[allow(dead_code)]
    pub fn construct_funding_transaction(
        &self,
        output_script: &Script,
        value: u64,
        target_blocks: usize,
    ) -> Result<Transaction, Error> {
        let wallet = self.inner.lock().unwrap();

        let mut tx_builder = wallet.build_tx();
        let fee_rate = wallet.client().estimate_fee(target_blocks)?;

        tx_builder
            .add_recipient(output_script.clone(), value)
            .fee_rate(fee_rate)
            .enable_rbf();

        let (mut psbt, _tx_details) = tx_builder.finish()?;

        let _finalized = wallet.sign(&mut psbt, SignOptions::default())?;

        Ok(psbt.extract_tx())
    }

    #[allow(dead_code)]
    pub fn get_balance(&self) -> Result<u64, Error> {
        let wallet = self.inner.lock().unwrap();
        wallet.get_balance().map_err(Error::Bdk)
    }

    #[allow(dead_code)]
    pub fn get_wallet(&self) -> MutexGuard<Wallet<B, D>> {
        self.inner.lock().unwrap()
    }

    #[allow(dead_code)]
    fn sync(&self) -> Result<(), Error> {
        let wallet = self.inner.lock().unwrap();
        wallet.sync(noop_progress(), None)?;
        Ok(())
    }
}

impl<B, D> From<Wallet<B, D>> for LightningWallet<B, D>
where
    B: Blockchain,
    D: BatchDatabase,
{
    fn from(wallet: Wallet<B, D>) -> Self {
        Self::new(wallet)
    }
}

impl<B, D> FeeEstimator for LightningWallet<B, D>
where
    B: Blockchain,
    D: BatchDatabase,
{
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        let wallet = self.inner.lock().unwrap();

        let target_blocks = match confirmation_target {
            ConfirmationTarget::Background => 6,
            ConfirmationTarget::Normal => 3,
            ConfirmationTarget::HighPriority => 1,
        };

        let estimate = wallet
            .client()
            .estimate_fee(target_blocks)
            .unwrap_or_default();
        let sats_per_vbyte = estimate.as_sat_vb() as u32;
        sats_per_vbyte * 253
    }
}

impl<B, D> BroadcasterInterface for LightningWallet<B, D>
where
    B: Blockchain,
    D: BatchDatabase,
{
    fn broadcast_transaction(&self, tx: &Transaction) {
        let wallet = self.inner.lock().unwrap();
        let _result = wallet.client().broadcast(tx);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
