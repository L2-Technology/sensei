use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20221003_000001_create_cluster_nodes_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(ClusterNode::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClusterNode::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClusterNode::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClusterNode::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ClusterNode::NodeId).string().not_null())
                    .col(ColumnDef::new(ClusterNode::Host).string().not_null())
                    .col(ColumnDef::new(ClusterNode::Port).integer().not_null())
                    .col(ColumnDef::new(ClusterNode::MacaroonHex).string().not_null())
                    .col(ColumnDef::new(ClusterNode::Label).string())
                    .col(ColumnDef::new(ClusterNode::Pubkey).string().not_null())
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(ClusterNode::Table)
                    .name("idx-cn-nodeid-pubkey")
                    .col(ClusterNode::NodeId)
                    .col(ClusterNode::Pubkey)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(ClusterNode::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum ClusterNode {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Host,
    Port,
    MacaroonHex,
    Label,
    Pubkey,
}
