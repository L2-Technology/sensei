use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220424_000001_create_access_tokens_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccessToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccessToken::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccessToken::Token)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AccessToken::Name).string().not_null())
                    .col(ColumnDef::new(AccessToken::Scope).text().not_null())
                    .col(ColumnDef::new(AccessToken::SingleUse).boolean().not_null())
                    .col(
                        ColumnDef::new(AccessToken::ExpiresAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AccessToken::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AccessToken::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut stmt = Table::drop();
        stmt.table(AccessToken::Table);
        manager.drop_table(stmt).await
    }
}

#[derive(Iden)]
enum AccessToken {
    Table,
    Id,
    Name,
    Scope,
    Token,
    ExpiresAt,
    SingleUse,
    CreatedAt,
    UpdatedAt,
}
