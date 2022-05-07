use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220428_000003_create_transactions_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transaction::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Transaction::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transaction::NodeId).string().not_null())
                    .col(ColumnDef::new(Transaction::Txid).string().not_null())
                    .col(ColumnDef::new(Transaction::RawTx).binary())
                    .col(ColumnDef::new(Transaction::Received).big_integer())
                    .col(ColumnDef::new(Transaction::Sent).big_integer())
                    .col(ColumnDef::new(Transaction::Fee).big_integer())
                    .col(ColumnDef::new(Transaction::ConfirmationTime).binary())
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(Transaction::Table)
                    .name("idx-nodeid-txid")
                    .col(Transaction::NodeId)
                    .col(Transaction::Txid)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(Transaction::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum Transaction {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Txid,
    Received,
    Sent,
    Fee,
    ConfirmationTime,
    RawTx,
}
