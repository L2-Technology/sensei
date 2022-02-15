use bdk::{
    database::{BatchOperations, Database, SqliteDatabase},
    BlockTime, KeychainKind, LocalUtxo, TransactionDetails,
};
use bitcoin::{Block, BlockHeader, OutPoint, Script, Transaction, TxOut, Txid};
use lightning::chain::Listen;

use crate::database::node::NodeDatabase;

#[derive(Clone)]
pub struct ListenerDatabase {
    bdk_db_path: String,
    node_db_path: String,
}

impl ListenerDatabase {
    pub fn new(bdk_db_path: String, node_db_path: String) -> Self {
        Self {
            bdk_db_path,
            node_db_path,
        }
    }

    pub fn process_mempool_tx(&self, tx: &Transaction) {
        let mut database = SqliteDatabase::new(self.bdk_db_path.clone());
        let mut internal_max_deriv = None;
        let mut external_max_deriv = None;

        self.process_tx(
            tx,
            &mut database,
            None,
            None,
            &mut internal_max_deriv,
            &mut external_max_deriv,
        );

        let current_ext = database
            .get_last_index(KeychainKind::External)
            .unwrap()
            .unwrap_or(0);
        let first_ext_new = external_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_ext_new > current_ext {
            database
                .set_last_index(KeychainKind::External, first_ext_new)
                .unwrap();
        }

        let current_int = database
            .get_last_index(KeychainKind::Internal)
            .unwrap()
            .unwrap_or(0);
        let first_int_new = internal_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_int_new > current_int {
            database
                .set_last_index(KeychainKind::Internal, first_int_new)
                .unwrap();
        }
    }

    pub fn process_tx(
        &self,
        tx: &Transaction,
        database: &mut SqliteDatabase,
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
        for (i, input) in tx.input.iter().enumerate() {
            if let Some(previous_output) = database
                .get_previous_output(&input.previous_output)
                .unwrap()
            {
                inputs_sum += previous_output.value;

                if database.is_mine(&previous_output.script_pubkey).unwrap() {
                    outgoing += previous_output.value;

                    database.del_utxo(&input.previous_output).unwrap();
                }
            }
        }

        for (i, output) in tx.output.iter().enumerate() {
            // to compute the fees later
            outputs_sum += output.value;

            // this output is ours, we have a path to derive it
            if let Some((keychain, child)) = database
                .get_path_from_script_pubkey(&output.script_pubkey)
                .unwrap()
            {
                database
                    .set_utxo(&LocalUtxo {
                        outpoint: OutPoint::new(tx.txid(), i as u32),
                        txout: output.clone(),
                        keychain,
                    })
                    .unwrap();
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
                verified: true,
                fee: Some(inputs_sum.saturating_sub(outputs_sum)),
            };

            database.set_tx(&tx).unwrap();
        }
    }
}

impl Listen for ListenerDatabase {
    fn block_connected(&self, block: &Block, height: u32) {
        let mut database = SqliteDatabase::new(self.bdk_db_path.clone());
        let mut internal_max_deriv = None;
        let mut external_max_deriv = None;

        // iterate all transactions in the block, looking for ones we care about
        for tx in &block.txdata {
            self.process_tx(
                tx,
                &mut database,
                Some(height),
                Some(block.header.time.into()),
                &mut internal_max_deriv,
                &mut external_max_deriv,
            )
        }

        let current_ext = database
            .get_last_index(KeychainKind::External)
            .unwrap()
            .unwrap_or(0);
        let first_ext_new = external_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_ext_new > current_ext {
            database
                .set_last_index(KeychainKind::External, first_ext_new)
                .unwrap();
        }

        let current_int = database
            .get_last_index(KeychainKind::Internal)
            .unwrap()
            .unwrap_or(0);
        let first_int_new = internal_max_deriv.map(|x| x + 1).unwrap_or(0);
        if first_int_new > current_int {
            database
                .set_last_index(KeychainKind::Internal, first_int_new)
                .unwrap();
        }

        // TODO: there's probably a bug here.
        //       need to atomicly update bdk database and this last_sync
        let mut node_database = NodeDatabase::new(self.node_db_path.clone());
        node_database.update_last_sync(block.block_hash()).unwrap();
    }

    fn block_disconnected(&self, header: &BlockHeader, height: u32) {
        let mut database = SqliteDatabase::new(self.bdk_db_path.clone());
        let mut deleted_txids = vec![];

        // delete all transactions with this height
        for details in database.iter_txs(false).unwrap() {
            match details.confirmation_time {
                Some(c) if c.height < height => continue,
                _ => {
                    database.del_tx(&details.txid, false).unwrap();
                    deleted_txids.push(details.txid)
                }
            };
        }

        // delete all utxos from the deleted txs
        if deleted_txids.len() > 0 {
            for utxo in database.iter_utxos().unwrap() {
                if deleted_txids.contains(&utxo.outpoint.txid) {
                    database.del_utxo(&utxo.outpoint).unwrap();
                }
            }
        }

        // TODO: update the keychain indexes?
        //

        // TODO: there's probably a bug here.
        //       need to atomicly update bdk database and this last_sync
        let mut node_database = NodeDatabase::new(self.node_db_path.clone());
        node_database
            .update_last_sync(header.prev_blockhash)
            .unwrap();
    }
}

pub(crate) trait DatabaseUtils: Database {
    fn is_mine(&self, script: &Script) -> Result<bool, bdk::Error> {
        self.get_path_from_script_pubkey(script)
            .map(|o| o.is_some())
    }

    fn get_raw_tx_or<D>(&self, txid: &Txid, default: D) -> Result<Option<Transaction>, bdk::Error>
    where
        D: FnOnce() -> Result<Option<Transaction>, bdk::Error>,
    {
        self.get_tx(txid, true)?
            .map(|t| t.transaction)
            .flatten()
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
