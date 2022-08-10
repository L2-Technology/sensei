use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220808_000001_create_users_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(User::Role).small_integer().not_null())
                    .col(
                        ColumnDef::new(User::Username)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::HashedPassword).string().not_null())
                    .col(ColumnDef::new(User::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(User::UpdatedAt).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut stmt = Table::drop();
        stmt.table(User::Table);
        manager.drop_table(stmt).await
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    Role,
    Username,
    HashedPassword,
    CreatedAt,
    UpdatedAt,
}
