use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220424_000003_create_macaroons_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(Macaroon::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Macaroon::Id)
                            .string()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Macaroon::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Macaroon::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Macaroon::NodeId).string().not_null())
                    .col(
                        ColumnDef::new(Macaroon::EncryptedMacaroon)
                            .binary()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await;
        manager
            .create_index(
                Index::create()
                    .name("idx-nodeid")
                    .table(Macaroon::Table)
                    .col(Macaroon::NodeId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut stmt = Table::drop();
        stmt.table(Macaroon::Table);
        manager.drop_table(stmt).await
    }
}

#[derive(Iden)]
enum Macaroon {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    EncryptedMacaroon,
}
