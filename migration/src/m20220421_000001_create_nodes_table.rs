use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220421_000001_create_nodes_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Node::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Node::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Node::Role).small_integer().not_null())
                    .col(
                        ColumnDef::new(Node::Username)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Node::Alias).string().not_null())
                    .col(ColumnDef::new(Node::Network).string().not_null())
                    .col(ColumnDef::new(Node::ListenAddr).string().not_null())
                    .col(ColumnDef::new(Node::ListenPort).integer().not_null())
                    .col(ColumnDef::new(Node::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Node::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Node::Status).small_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut stmt = Table::drop();
        stmt.table(Node::Table);
        manager.drop_table(stmt).await
    }
}

#[derive(Iden)]
enum Node {
    Id,
    Table,
    Role,
    Username,
    Alias,
    Network,
    ListenAddr,
    ListenPort,
    CreatedAt,
    UpdatedAt,
    Status,
}
