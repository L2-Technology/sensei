use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220428_000004_create_keychains_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(Keychain::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Keychain::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Keychain::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Keychain::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Keychain::NodeId).string().not_null())
                    .col(ColumnDef::new(Keychain::Keychain).string().not_null())
                    .col(ColumnDef::new(Keychain::LastDerivationIndex).integer())
                    .col(ColumnDef::new(Keychain::Checksum).binary())
                    .to_owned(),
            )
            .await;

        // Q: With only two Keychain values is it really useful to add index?
        // Q: Should this whole table just be in the kv_store?
        manager
            .create_index(
                Index::create()
                    .table(Keychain::Table)
                    .name("idx-nodeid-keychain")
                    .col(Keychain::NodeId)
                    .col(Keychain::Keychain)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(Keychain::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum Keychain {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Keychain,
    LastDerivationIndex,
    Checksum,
}
