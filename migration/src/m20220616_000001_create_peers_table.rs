use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220616_000001_create_peers_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(Peer::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Peer::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Peer::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Peer::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Peer::NodeId).string().not_null())
                    .col(ColumnDef::new(Peer::Pubkey).string().not_null())
                    .col(ColumnDef::new(Peer::Alias).string())
                    .col(ColumnDef::new(Peer::Label).string())
                    .col(ColumnDef::new(Peer::ZeroConf).boolean().not_null())
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(Peer::Table)
                    .name("idx-nodeid-pubkey")
                    .col(Peer::NodeId)
                    .col(Peer::Pubkey)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(Peer::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum Peer {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Pubkey,
    Label,
    Alias,
    ZeroConf,
}
