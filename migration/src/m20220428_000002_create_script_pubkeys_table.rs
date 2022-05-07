use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220428_000002_create_script_pubkeys_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(ScriptPubkey::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScriptPubkey::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ScriptPubkey::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScriptPubkey::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScriptPubkey::NodeId).string().not_null())
                    .col(ColumnDef::new(ScriptPubkey::Keychain).string().not_null())
                    .col(ColumnDef::new(ScriptPubkey::Child).integer().not_null())
                    .col(ColumnDef::new(ScriptPubkey::Script).string().not_null())
                    .to_owned(),
            )
            .await;

        let _res = manager
            .create_index(
                Index::create()
                    .table(ScriptPubkey::Table)
                    .name("idx-nodeid-keychain-child")
                    .col(ScriptPubkey::NodeId)
                    .col(ScriptPubkey::Keychain)
                    .col(ScriptPubkey::Child)
                    .unique()
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(ScriptPubkey::Table)
                    .name("idx-nodeid-script")
                    .col(ScriptPubkey::NodeId)
                    .col(ScriptPubkey::Script)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut drop_table_stmt = Table::drop();
        drop_table_stmt.table(ScriptPubkey::Table);
        manager.drop_table(drop_table_stmt).await
    }
}

#[derive(Iden)]
enum ScriptPubkey {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NodeId,
    Keychain,
    Child,
    Script,
}
