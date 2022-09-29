use crate::database::SenseiDatabase;
use bdk::database::{BatchDatabase, BatchOperations, Database, SyncTime};
use bdk::wallet::time;
use bdk::{BlockTime, KeychainKind, LocalUtxo, TransactionDetails};
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::{BlockHeader, OutPoint, Script, TxOut, Txid};
use entity::keychain::Entity as Keychain;
use entity::kv_store;
use entity::kv_store::Entity as KVStore;
use entity::script_pubkey;
use entity::script_pubkey::Entity as ScriptPubkey;
use entity::sea_orm::ActiveValue::{NotSet, Set};
use entity::sea_orm::ColumnTrait;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::ModelTrait;
use entity::sea_orm::QueryFilter;
use entity::sea_orm::{ActiveModelTrait, QueryOrder};
use entity::transaction;
use entity::transaction::Entity as Transaction;
use entity::utxo;
use entity::utxo::Entity as Utxo;
use entity::{hex_str, keychain, to_vec_unsafe};
use lightning::chain::transaction::TransactionData;
use lightning::chain::Listen;
use std::sync::Arc;

impl Listen for WalletDatabase {
    fn filtered_block_connected(
        &self,
        header: &BlockHeader,
        txdata: &TransactionData,
        height: u32,
    ) {
        let mut wallet_database = self.clone();

        let mut internal_max_deriv = None;
        let mut external_max_deriv = None;

        // iterate all transactions in the block, looking for ones we care about
        for (_, tx) in txdata {
            wallet_database.process_tx(
                tx,
                Some(height),
                Some(header.time.into()),
                &mut internal_max_deriv,
                &mut external_max_deriv,
            )
        }

        let current_ext = wallet_database
            .get_last_index(KeychainKind::External)
            .unwrap()
            .unwrap_or(0);
        let first_ext_new = external_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_ext_new > current_ext {
            wallet_database
                .set_last_index(KeychainKind::External, first_ext_new)
                .unwrap();
        }

        let current_int = wallet_database
            .get_last_index(KeychainKind::Internal)
            .unwrap()
            .unwrap_or(0);
        let first_int_new = internal_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_int_new > current_int {
            wallet_database
                .set_last_index(KeychainKind::Internal, first_int_new)
                .unwrap();
        }

        let timestamp = time::get_timestamp();

        let _res = wallet_database.set_sync_time(SyncTime {
            block_time: BlockTime { height, timestamp },
        });

        tokio::task::block_in_place(move || {
            wallet_database.tokio_handle.block_on(async move {
                wallet_database
                    .database
                    .create_or_update_last_onchain_wallet_sync(
                        wallet_database.node_id.clone(),
                        header.block_hash(),
                        height,
                        timestamp,
                    )
                    .await
                    .unwrap();
            });
        });
    }

    fn block_disconnected(&self, header: &BlockHeader, height: u32) {
        let mut wallet_database = self.clone();
        let mut deleted_txids = vec![];

        // delete all transactions with this height
        for details in wallet_database.iter_txs(false).unwrap() {
            match details.confirmation_time {
                Some(c) if c.height < height => continue,
                _ => {
                    wallet_database.del_tx(&details.txid, false).unwrap();
                    deleted_txids.push(details.txid)
                }
            };
        }

        // delete all utxos from the deleted txs
        if !deleted_txids.is_empty() {
            for utxo in wallet_database.iter_utxos().unwrap() {
                if deleted_txids.contains(&utxo.outpoint.txid) {
                    wallet_database.del_utxo(&utxo.outpoint).unwrap();
                }
            }
        }

        // TODO: update the keychain indexes?

        tokio::task::block_in_place(move || {
            wallet_database.tokio_handle.block_on(async move {
                wallet_database
                    .database
                    .create_or_update_last_onchain_wallet_sync(
                        wallet_database.node_id.clone(),
                        header.prev_blockhash,
                        height - 1,
                        time::get_timestamp(),
                    )
                    .await
                    .unwrap();
            });
        });
    }
}

#[derive(Clone)]
pub struct WalletDatabase {
    pub node_id: String,
    pub database: Arc<SenseiDatabase>,
    pub tokio_handle: tokio::runtime::Handle,
}

impl WalletDatabase {
    pub fn new(
        node_id: String,
        database: Arc<SenseiDatabase>,
        tokio_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            node_id,
            database,
            tokio_handle,
        }
    }

    pub fn process_mempool_tx(&mut self, tx: &bitcoin::Transaction) {
        let mut internal_max_deriv = None;
        let mut external_max_deriv = None;

        self.process_tx(
            tx,
            None,
            None,
            &mut internal_max_deriv,
            &mut external_max_deriv,
        );

        let current_ext = self
            .get_last_index(KeychainKind::External)
            .unwrap()
            .unwrap_or(0);
        let first_ext_new = external_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_ext_new > current_ext {
            self.set_last_index(KeychainKind::External, first_ext_new)
                .unwrap();
        }

        let current_int = self
            .get_last_index(KeychainKind::Internal)
            .unwrap()
            .unwrap_or(0);
        let first_int_new = internal_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_int_new > current_int {
            self.set_last_index(KeychainKind::Internal, first_int_new)
                .unwrap();
        }
    }

    pub fn process_tx(
        &mut self,
        tx: &bitcoin::Transaction,
        confirmation_height: Option<u32>,
        confirmation_time: Option<u64>,
        internal_max_deriv: &mut Option<u32>,
        external_max_deriv: &mut Option<u32>,
    ) {
        let mut incoming: u64 = 0;
        let mut outgoing: u64 = 0;

        let mut inputs_sum: u64 = 0;
        let mut outputs_sum: u64 = 0;

        // look for our own inputs
        for (_i, input) in tx.input.iter().enumerate() {
            if let Some(previous_output) = self.get_previous_output(&input.previous_output).unwrap()
            {
                inputs_sum += previous_output.value;

                if self.is_mine(&previous_output.script_pubkey).unwrap() {
                    outgoing += previous_output.value;

                    self.del_utxo(&input.previous_output).unwrap();
                }
            }
        }

        for (i, output) in tx.output.iter().enumerate() {
            // to compute the fees later
            outputs_sum += output.value;

            // this output is ours, we have a path to derive it
            if let Some((keychain, child)) = self
                .get_path_from_script_pubkey(&output.script_pubkey)
                .unwrap()
            {
                let txid = hex_str(&serialize(&tx.txid()));
                let existing_utxo = self.get_utxo(txid, i as i32).unwrap();
                if existing_utxo.is_none() {
                    self.set_utxo(&LocalUtxo {
                        outpoint: OutPoint::new(tx.txid(), i as u32),
                        txout: output.clone(),
                        keychain,
                        is_spent: false,
                    })
                    .unwrap();
                }
                incoming += output.value;

                // TODO: implement this

                if keychain == KeychainKind::Internal
                    && (internal_max_deriv.is_none() || child > internal_max_deriv.unwrap_or(0))
                {
                    *internal_max_deriv = Some(child);
                } else if keychain == KeychainKind::External
                    && (external_max_deriv.is_none() || child > external_max_deriv.unwrap_or(0))
                {
                    *external_max_deriv = Some(child);
                }
            }
        }

        if incoming > 0 || outgoing > 0 {
            let tx = TransactionDetails {
                txid: tx.txid(),
                transaction: Some(tx.clone()),
                received: incoming,
                sent: outgoing,
                confirmation_time: BlockTime::new(confirmation_height, confirmation_time),
                fee: Some(inputs_sum.saturating_sub(outputs_sum)),
            };
            self.set_tx(&tx).unwrap();
        }
    }

    pub fn insert_utxo(&self, utxo: utxo::ActiveModel) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                utxo.insert(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn insert_keychain(&self, keychain: keychain::ActiveModel) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                keychain
                    .insert(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn insert_kv_store(&self, kv_store: kv_store::ActiveModel) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                kv_store
                    .insert(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn insert_script_pubkey(
        &self,
        script_pubkey: script_pubkey::ActiveModel,
    ) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                script_pubkey
                    .insert(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn insert_transaction(
        &self,
        transaction: transaction::ActiveModel,
    ) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                transaction
                    .insert(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn update_utxo(&self, utxo: utxo::ActiveModel) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                utxo.update(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn update_transaction(
        &self,
        transaction: transaction::ActiveModel,
    ) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                transaction
                    .update(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn update_script_pubkey(
        &self,
        script_pubkey: script_pubkey::ActiveModel,
    ) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                script_pubkey
                    .update(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn update_keychain(&self, keychain: keychain::ActiveModel) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                keychain
                    .update(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn update_kv_store(&self, kv_store: kv_store::ActiveModel) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                kv_store
                    .update(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn delete_utxo(&self, utxo: utxo::Model) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                utxo.delete(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn delete_transaction(&self, transaction: transaction::Model) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                transaction
                    .delete(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn delete_script_pubkey(
        &self,
        script_pubkey: script_pubkey::Model,
    ) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                script_pubkey
                    .delete(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn delete_keychain(&self, keychain: keychain::Model) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                keychain
                    .delete(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn delete_kv_store(&self, kv_store: kv_store::Model) -> Result<(), bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                kv_store
                    .delete(self.database.get_connection())
                    .await
                    .map(|_| ())
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn get_value(&self, key: &str) -> Result<Option<kv_store::Model>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                KVStore::find()
                    .filter(kv_store::Column::NodeId.eq(self.node_id.clone()))
                    .filter(kv_store::Column::K.eq(key))
                    .one(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn set_value(&self, key: &str, value: Vec<u8>) -> Result<(), bdk::Error> {
        match self.get_value(key)? {
            Some(entity) => {
                let mut entity: kv_store::ActiveModel = entity.into();
                entity.v = Set(value);
                self.update_kv_store(entity)
            }
            None => {
                let entity = kv_store::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    k: Set(key.to_string()),
                    v: Set(value),
                    ..Default::default()
                };
                self.insert_kv_store(entity)
            }
        }
    }

    pub fn get_utxos(&self) -> Result<Vec<bdk::LocalUtxo>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                Utxo::find()
                    .filter(utxo::Column::NodeId.eq(self.node_id.clone()))
                    .all(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))?
                    .iter()
                    .map(|utxo| utxo.to_local_utxo())
                    .collect()
            })
        })
    }

    pub fn get_raw_transactions(&self) -> Result<Vec<bitcoin::Transaction>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                Ok(Transaction::find()
                    .filter(transaction::Column::NodeId.eq(self.node_id.clone()))
                    .order_by_desc(transaction::Column::CreatedAt)
                    .all(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))?
                    .iter()
                    .filter_map(|tx| {
                        tx.raw_tx.as_ref().map(|raw_tx| {
                            let tx: bitcoin::Transaction = deserialize(raw_tx).unwrap();
                            tx
                        })
                    })
                    .collect())
            })
        })
    }

    pub fn get_transactions(
        &self,
        _include_raw: bool,
    ) -> Result<Vec<bdk::TransactionDetails>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                Transaction::find()
                    .filter(transaction::Column::NodeId.eq(self.node_id.clone()))
                    .order_by_desc(transaction::Column::CreatedAt)
                    .all(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))?
                    .iter()
                    .map(|tx| tx.to_transaction_details())
                    .collect()
            })
        })
    }

    pub fn get_script_pubkeys(
        &self,
        keychain: Option<String>,
    ) -> Result<Vec<bitcoin::Script>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                let query = ScriptPubkey::find()
                    .filter(script_pubkey::Column::NodeId.eq(self.node_id.clone()));

                let query = match keychain {
                    Some(keychain) => query.filter(script_pubkey::Column::Keychain.eq(keychain)),
                    None => query,
                };

                Ok(query
                    .all(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))?
                    .iter()
                    .map(|script_pubkey| {
                        deserialize(&to_vec_unsafe(&script_pubkey.script)).unwrap()
                    })
                    .collect())
            })
        })
    }

    pub fn get_script_pubkey_by_path(
        &self,
        keychain: String,
        child: i32,
    ) -> Result<Option<script_pubkey::Model>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                ScriptPubkey::find()
                    .filter(script_pubkey::Column::NodeId.eq(self.node_id.clone()))
                    .filter(script_pubkey::Column::Keychain.eq(keychain))
                    .filter(script_pubkey::Column::Child.eq(child))
                    .one(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn get_script_pubkey_by_script(
        &self,
        script: String,
    ) -> Result<Option<script_pubkey::Model>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                ScriptPubkey::find()
                    .filter(script_pubkey::Column::NodeId.eq(self.node_id.clone()))
                    .filter(script_pubkey::Column::Script.eq(script))
                    .one(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn get_utxo(&self, txid: String, vout: i32) -> Result<Option<utxo::Model>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                Utxo::find()
                    .filter(utxo::Column::Txid.eq(txid))
                    .filter(utxo::Column::Vout.eq(vout))
                    .one(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn get_transaction(&self, txid: String) -> Result<Option<transaction::Model>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                Transaction::find()
                    .filter(transaction::Column::NodeId.eq(self.node_id.clone()))
                    .filter(transaction::Column::Txid.eq(txid))
                    .one(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }

    pub fn get_keychain(&self, keychain: String) -> Result<Option<keychain::Model>, bdk::Error> {
        tokio::task::block_in_place(move || {
            self.tokio_handle.block_on(async move {
                Keychain::find()
                    .filter(keychain::Column::NodeId.eq(self.node_id.clone()))
                    .filter(keychain::Column::Keychain.eq(keychain))
                    .one(self.database.get_connection())
                    .await
                    .map_err(|e| bdk::Error::Generic(e.to_string()))
            })
        })
    }
}

impl BatchOperations for WalletDatabase {
    fn set_script_pubkey(
        &mut self,
        script: &bitcoin::Script,
        keychain: bdk::KeychainKind,
        child: u32,
    ) -> Result<(), bdk::Error> {
        let script = hex_str(&serialize(script));
        let keychain = serde_json::to_string(&keychain)?;
        let child: i32 = child.try_into().unwrap();

        match self.get_script_pubkey_by_path(keychain.clone(), child)? {
            Some(script_pubkey) => {
                let mut script_pubkey: script_pubkey::ActiveModel = script_pubkey.into();
                script_pubkey.script = Set(script);
                self.update_script_pubkey(script_pubkey)
            }
            None => {
                let script_pubkey = script_pubkey::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    keychain: Set(keychain),
                    child: Set(child),
                    script: Set(script),
                    ..Default::default()
                };
                self.insert_script_pubkey(script_pubkey)
            }
        }
    }

    fn set_utxo(&mut self, utxo: &bdk::LocalUtxo) -> Result<(), bdk::Error> {
        let value: i64 = utxo.txout.value.try_into().unwrap();
        let keychain = serde_json::to_string(&utxo.keychain)?;
        let vout: i32 = utxo.outpoint.vout.try_into().unwrap();
        let txid = hex_str(&serialize(&utxo.outpoint.txid));
        let script = hex_str(&serialize(&utxo.txout.script_pubkey));
        let is_spent = utxo.is_spent;

        match self.get_utxo(txid.clone(), vout)? {
            Some(utxo) => {
                let mut utxo: utxo::ActiveModel = utxo.into();
                utxo.value = Set(value);
                utxo.keychain = Set(keychain);
                utxo.script = Set(script);
                utxo.is_spent = Set(is_spent);
                self.update_utxo(utxo)
            }
            None => {
                let utxo = utxo::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    keychain: Set(keychain),
                    txid: Set(txid),
                    vout: Set(vout),
                    script: Set(script),
                    value: Set(value),
                    is_spent: Set(is_spent),
                    ..Default::default()
                };
                self.insert_utxo(utxo)
            }
        }
    }

    fn set_raw_tx(&mut self, btc_transaction: &bitcoin::Transaction) -> Result<(), bdk::Error> {
        let txid = hex_str(&serialize(&btc_transaction.txid()));
        match self.get_transaction(txid)? {
            Some(transaction) => {
                let mut transaction: transaction::ActiveModel = transaction.into();
                transaction.raw_tx = Set(Some(serialize(btc_transaction)));
                self.update_transaction(transaction)
            }
            None => {
                let transaction = transaction::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    raw_tx: Set(Some(serialize(btc_transaction))),
                    ..Default::default()
                };
                self.insert_transaction(transaction)
            }
        }
    }

    fn set_tx(&mut self, transaction: &bdk::TransactionDetails) -> Result<(), bdk::Error> {
        let txid = hex_str(&serialize(&transaction.txid));
        let received: i64 = transaction.received.try_into().unwrap();
        let sent: i64 = transaction.sent.try_into().unwrap();
        let fee: Option<i64> = transaction.fee.map(|fee| fee.try_into().unwrap());
        let raw_tx = transaction.transaction.as_ref().map(serialize);
        let confirmation_time = transaction
            .confirmation_time
            .as_ref()
            .map(|ct| serde_json::to_vec(ct).unwrap());

        match self.get_transaction(txid.clone())? {
            Some(transaction) => {
                let mut transaction: transaction::ActiveModel = transaction.into();
                // TODO (maybe): Set raw_tx if not set? Not sure if that ever happens
                transaction.received = Set(Some(received));
                transaction.sent = Set(Some(sent));
                transaction.fee = Set(fee);
                transaction.confirmation_time = Set(confirmation_time);
                self.update_transaction(transaction)
            }
            None => {
                let transaction = transaction::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    raw_tx: Set(raw_tx),
                    txid: Set(txid),
                    received: Set(Some(received)),
                    sent: Set(Some(sent)),
                    fee: Set(fee),
                    confirmation_time: Set(confirmation_time),
                    ..Default::default()
                };
                self.insert_transaction(transaction)
            }
        }
    }

    fn set_last_index(
        &mut self,
        keychain: bdk::KeychainKind,
        value: u32,
    ) -> Result<(), bdk::Error> {
        let keychain = serde_json::to_string(&keychain)?;
        let last_derivation_index: i32 = value.try_into().unwrap();

        match self.get_keychain(keychain.clone())? {
            Some(keychain) => {
                let mut keychain: keychain::ActiveModel = keychain.into();
                keychain.last_derivation_index = Set(last_derivation_index);
                self.update_keychain(keychain)
            }
            None => {
                let keychain = keychain::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    keychain: Set(keychain),
                    last_derivation_index: Set(last_derivation_index),
                    ..Default::default()
                };
                self.insert_keychain(keychain)
            }
        }
    }

    fn set_sync_time(&mut self, sync_time: bdk::database::SyncTime) -> Result<(), bdk::Error> {
        let last_sync_key = format!("{}/chain/last_sync", self.node_id.clone());
        let last_sync_value = serde_json::to_vec(&sync_time)?;
        self.set_value(&last_sync_key, last_sync_value)
    }

    fn del_script_pubkey_from_path(
        &mut self,
        keychain: bdk::KeychainKind,
        child: u32,
    ) -> Result<Option<bitcoin::Script>, bdk::Error> {
        let keychain = serde_json::to_string(&keychain)?;
        let child: i32 = child.try_into().unwrap();

        match self.get_script_pubkey_by_path(keychain, child)? {
            Some(script_pubkey) => {
                let script: bitcoin::Script = deserialize(&to_vec_unsafe(&script_pubkey.script))?;
                self.delete_script_pubkey(script_pubkey)
                    .map(|_| Some(script))
            }
            None => Ok(None),
        }
    }

    fn del_path_from_script_pubkey(
        &mut self,
        script: &bitcoin::Script,
    ) -> Result<Option<(bdk::KeychainKind, u32)>, bdk::Error> {
        let script = hex_str(&serialize(script));

        match self.get_script_pubkey_by_script(script)? {
            Some(script_pubkey) => {
                let keychain: bdk::KeychainKind = serde_json::from_str(&script_pubkey.keychain)?;
                let child: u32 = script_pubkey.child as u32;
                self.delete_script_pubkey(script_pubkey)
                    .map(|_| Some((keychain, child)))
            }
            None => Ok(None),
        }
    }

    fn del_utxo(
        &mut self,
        outpoint: &bitcoin::OutPoint,
    ) -> Result<Option<bdk::LocalUtxo>, bdk::Error> {
        let vout: i32 = outpoint.vout.try_into().unwrap();
        let txid = hex_str(&serialize(&outpoint.txid));

        match self.get_utxo(txid, vout)? {
            Some(utxo) => {
                let value = utxo.value as u64;
                let keychain: bdk::KeychainKind = serde_json::from_str(&utxo.keychain)?;
                let script_pubkey: bitcoin::Script = deserialize(&to_vec_unsafe(&utxo.script))?;
                let is_spent = utxo.is_spent;
                self.delete_utxo(utxo).map(|_| {
                    Some(bdk::LocalUtxo {
                        outpoint: *outpoint,
                        txout: bitcoin::TxOut {
                            value,
                            script_pubkey,
                        },
                        keychain,
                        is_spent,
                    })
                })
            }
            None => Ok(None),
        }
    }

    fn del_raw_tx(
        &mut self,
        txid: &bitcoin::Txid,
    ) -> Result<Option<bitcoin::Transaction>, bdk::Error> {
        let txid = hex_str(&serialize(txid));

        match self.get_transaction(txid)? {
            Some(transaction) => {
                let raw_tx: Option<bitcoin::Transaction> = transaction
                    .raw_tx
                    .as_ref()
                    .map(|raw_tx| deserialize(raw_tx).unwrap());
                let mut transaction: transaction::ActiveModel = transaction.into();
                transaction.raw_tx = Set(None);
                self.update_transaction(transaction).map(|_| raw_tx)
            }
            None => Ok(None),
        }
    }

    fn del_tx(
        &mut self,
        txid: &bitcoin::Txid,
        include_raw: bool,
    ) -> Result<Option<bdk::TransactionDetails>, bdk::Error> {
        let txid = hex_str(&serialize(txid));

        match self.get_transaction(txid)? {
            Some(transaction) => {
                let details = transaction.to_transaction_details()?;
                if include_raw {
                    self.delete_transaction(transaction).map(|_| Some(details))
                } else {
                    let mut transaction: transaction::ActiveModel = transaction.into();
                    transaction.received = NotSet;
                    transaction.sent = NotSet;
                    transaction.fee = Set(None);
                    transaction.confirmation_time = Set(None);
                    self.update_transaction(transaction).map(|_| Some(details))
                }
            }
            None => Ok(None),
        }
    }

    fn del_last_index(&mut self, keychain: bdk::KeychainKind) -> Result<Option<u32>, bdk::Error> {
        let keychain = serde_json::to_string(&keychain)?;
        match self.get_keychain(keychain)? {
            Some(keychain) => {
                let last_derivation_index: u32 = keychain.last_derivation_index as u32;
                let mut keychain: keychain::ActiveModel = keychain.into();
                keychain.last_derivation_index = NotSet;
                self.update_keychain(keychain)
                    .map(|_| Some(last_derivation_index))
            }
            None => Ok(None),
        }
    }

    fn del_sync_time(&mut self) -> Result<Option<bdk::database::SyncTime>, bdk::Error> {
        let last_sync_key = format!("{}/chain/last_sync", self.node_id.clone());
        match self.get_value(&last_sync_key)? {
            Some(last_sync_entity) => {
                let sync_time: bdk::database::SyncTime =
                    serde_json::from_slice(&last_sync_entity.v)?;
                self.delete_kv_store(last_sync_entity)
                    .map(|_| Some(sync_time))
            }
            None => Ok(None),
        }
    }
}

impl Database for WalletDatabase {
    fn check_descriptor_checksum<B: AsRef<[u8]>>(
        &mut self,
        keychain: bdk::KeychainKind,
        bytes: B,
    ) -> Result<(), bdk::Error> {
        let keychain = serde_json::to_string(&keychain)?;

        match self.get_keychain(keychain.clone())? {
            Some(keychain) => {
                if keychain.checksum == bytes.as_ref().to_vec() {
                    Ok(())
                } else {
                    Err(bdk::Error::ChecksumMismatch)
                }
            }
            None => {
                let keychain = keychain::ActiveModel {
                    node_id: Set(self.node_id.clone()),
                    keychain: Set(keychain),
                    last_derivation_index: Set(0),
                    checksum: Set(bytes.as_ref().to_vec()),
                    ..Default::default()
                };

                self.insert_keychain(keychain)
            }
        }
    }

    fn iter_script_pubkeys(
        &self,
        keychain: Option<bdk::KeychainKind>,
    ) -> Result<Vec<bitcoin::Script>, bdk::Error> {
        let keychain = keychain.map(|keychain| serde_json::to_string(&keychain).unwrap());
        self.get_script_pubkeys(keychain)
    }

    fn iter_utxos(&self) -> Result<Vec<bdk::LocalUtxo>, bdk::Error> {
        self.get_utxos()
    }

    fn iter_raw_txs(&self) -> Result<Vec<bitcoin::Transaction>, bdk::Error> {
        self.get_raw_transactions()
    }

    fn iter_txs(&self, include_raw: bool) -> Result<Vec<bdk::TransactionDetails>, bdk::Error> {
        self.get_transactions(include_raw)
    }

    fn get_script_pubkey_from_path(
        &self,
        keychain: bdk::KeychainKind,
        child: u32,
    ) -> Result<Option<bitcoin::Script>, bdk::Error> {
        let keychain = serde_json::to_string(&keychain)?;
        let child: i32 = child.try_into().unwrap();
        Ok(self
            .get_script_pubkey_by_path(keychain, child)?
            .map(|keychain| deserialize(&to_vec_unsafe(&keychain.script)).unwrap()))
    }

    fn get_path_from_script_pubkey(
        &self,
        script: &bitcoin::Script,
    ) -> Result<Option<(bdk::KeychainKind, u32)>, bdk::Error> {
        let script = hex_str(&serialize(script));
        self.get_script_pubkey_by_script(script).map(|keychain| {
            keychain.map(|keychain| {
                let child: u32 = keychain.child as u32;
                let keychain: bdk::KeychainKind = serde_json::from_str(&keychain.keychain).unwrap();
                (keychain, child)
            })
        })
    }

    fn get_utxo(&self, outpoint: &bitcoin::OutPoint) -> Result<Option<bdk::LocalUtxo>, bdk::Error> {
        let vout: i32 = outpoint.vout.try_into().unwrap();
        let txid = hex_str(&serialize(&outpoint.txid));
        self.get_utxo(txid, vout)
            .map(|utxo| utxo.map(|utxo| utxo.to_local_utxo().unwrap()))
    }

    fn get_raw_tx(&self, txid: &bitcoin::Txid) -> Result<Option<bitcoin::Transaction>, bdk::Error> {
        let txid = hex_str(&serialize(txid));
        self.get_transaction(txid).map(|tx| match tx {
            Some(tx) => tx.raw_tx.map(|raw_tx| deserialize(&raw_tx).unwrap()),
            None => None,
        })
    }

    fn get_tx(
        &self,
        txid: &bitcoin::Txid,
        _include_raw: bool,
    ) -> Result<Option<bdk::TransactionDetails>, bdk::Error> {
        let txid = hex_str(&serialize(txid));
        self.get_transaction(txid)
            .map(|tx| tx.map(|tx| tx.to_transaction_details().unwrap()))
    }

    fn get_last_index(&self, keychain: bdk::KeychainKind) -> Result<Option<u32>, bdk::Error> {
        let keychain = serde_json::to_string(&keychain)?;
        Ok(self
            .get_keychain(keychain)?
            .map(|keychain| keychain.last_derivation_index as u32))
    }

    fn get_sync_time(&self) -> Result<Option<bdk::database::SyncTime>, bdk::Error> {
        let last_sync_key = format!("{}/chain/last_sync", self.node_id.clone());
        self.get_value(&last_sync_key)
            .map(|entry| entry.map(|entry| serde_json::from_slice(&entry.v).unwrap()))
    }

    fn increment_last_index(&mut self, keychain: bdk::KeychainKind) -> Result<u32, bdk::Error> {
        match self.get_last_index(keychain)? {
            Some(index) => {
                self.set_last_index(keychain, index + 1)?;
                Ok(index + 1)
            }
            None => {
                self.set_last_index(keychain, 0)?;
                Ok(0)
            }
        }
    }
}

// TODO: actually implement batch operations?
//       just wrap in a transaction? not quite what we want I think
impl BatchDatabase for WalletDatabase {
    type Batch = WalletDatabase;

    fn begin_batch(&self) -> Self::Batch {
        self.clone()
    }

    fn commit_batch(&mut self, _batch: Self::Batch) -> Result<(), bdk::Error> {
        Ok(())
    }
}

trait DatabaseUtils: Database {
    fn is_mine(&self, script: &Script) -> Result<bool, bdk::Error> {
        self.get_path_from_script_pubkey(script)
            .map(|o| o.is_some())
    }

    fn get_raw_tx_or<D>(
        &self,
        txid: &Txid,
        default: D,
    ) -> Result<Option<bitcoin::Transaction>, bdk::Error>
    where
        D: FnOnce() -> Result<Option<bitcoin::Transaction>, bdk::Error>,
    {
        self.get_tx(txid, true)?
            .and_then(|t| t.transaction)
            .map_or_else(default, |t| Ok(Some(t)))
    }

    fn get_previous_output(&self, outpoint: &OutPoint) -> Result<Option<TxOut>, bdk::Error> {
        self.get_raw_tx(&outpoint.txid)?
            .map(|previous_tx| {
                if outpoint.vout as usize >= previous_tx.output.len() {
                    Err(bdk::Error::InvalidOutpoint(*outpoint))
                } else {
                    Ok(previous_tx.output[outpoint.vout as usize].clone())
                }
            })
            .transpose()
    }
}

impl<T: Database> DatabaseUtils for T {}
