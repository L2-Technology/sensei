use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220424_000005_create_kv_store_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(KvStore::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(KvStore::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(KvStore::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(KvStore::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(KvStore::NodeId).string().not_null())
                    .col(ColumnDef::new(KvStore::K).string().not_null())
                    .col(ColumnDef::new(KvStore::V).binary().not_null())
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(KvStore::Table)
                    .name("idx-nodeid-k")
                    .col(KvStore::NodeId)
                    .col(KvStore::K)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(KvStore::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum KvStore {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    K,
    V,
}
