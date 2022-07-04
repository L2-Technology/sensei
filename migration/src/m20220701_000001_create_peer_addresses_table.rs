use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220701_000001_create_peer_addresses_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(PeerAddress::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PeerAddress::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PeerAddress::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PeerAddress::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PeerAddress::LastConnectedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PeerAddress::NodeId).string().not_null())
                    .col(ColumnDef::new(PeerAddress::Pubkey).string().not_null())
                    .col(ColumnDef::new(PeerAddress::Address).binary().not_null())
                    .col(
                        ColumnDef::new(PeerAddress::Source)
                            .small_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(PeerAddress::Table)
                    .name("idx-addr-nodeid-pubkey")
                    .col(PeerAddress::NodeId)
                    .col(PeerAddress::Pubkey)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(PeerAddress::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum PeerAddress {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    LastConnectedAt,
    NodeId,
    Pubkey,
    Address,
    Source,
}
