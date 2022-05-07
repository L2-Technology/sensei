pub use sea_schema::migration::prelude::*;

mod m20220421_000001_create_nodes_table;
mod m20220424_000001_create_access_tokens_table;
mod m20220424_000002_create_macaroons_table;
mod m20220424_000003_create_payments_table;
mod m20220424_000004_create_kv_store_table;
mod m20220428_000001_create_utxos_table;
mod m20220428_000002_create_script_pubkeys_table;
mod m20220428_000003_create_transactions_table;
mod m20220428_000004_create_keychains_table;

pub struct Migrator;

// Note: migrations should be sorted in chronological order
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220421_000001_create_nodes_table::Migration),
            Box::new(m20220424_000001_create_access_tokens_table::Migration),
            Box::new(m20220424_000002_create_macaroons_table::Migration),
            Box::new(m20220424_000003_create_payments_table::Migration),
            Box::new(m20220424_000004_create_kv_store_table::Migration),
            Box::new(m20220428_000001_create_utxos_table::Migration),
            Box::new(m20220428_000002_create_script_pubkeys_table::Migration),
            Box::new(m20220428_000003_create_transactions_table::Migration),
            Box::new(m20220428_000004_create_keychains_table::Migration),
        ]
    }
}
