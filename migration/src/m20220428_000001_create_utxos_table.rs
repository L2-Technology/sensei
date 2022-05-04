use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220428_000001_create_utxos_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(Utxo::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Utxo::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Utxo::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Utxo::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Utxo::NodeId).string().not_null())
                    .col(ColumnDef::new(Utxo::Value).big_integer().not_null())
                    .col(ColumnDef::new(Utxo::Keychain).string().not_null())
                    .col(ColumnDef::new(Utxo::Vout).integer().not_null())
                    .col(ColumnDef::new(Utxo::Txid).string().not_null())
                    .col(ColumnDef::new(Utxo::Script).string().not_null())
                    .col(ColumnDef::new(Utxo::IsSpent).boolean().not_null())
                    .to_owned(),
            )
            .await;

        let _res = manager
            .create_index(
                Index::create()
                    .table(Utxo::Table)
                    .name("idx-txid-vout")
                    .col(Utxo::Txid)
                    .col(Utxo::Vout)
                    .unique()
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(Utxo::Table)
                    .name("idx-utxos-nodeid")
                    .col(Utxo::NodeId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(Utxo::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum Utxo {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Value,
    Keychain,
    Vout,
    Txid,
    Script,
    IsSpent,
}
