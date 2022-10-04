use sea_schema::migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220424_000004_create_payments_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _res = manager
            .create_table(
                Table::create()
                    .table(Payment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payment::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payment::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Payment::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Payment::NodeId).string().not_null())
                    .col(ColumnDef::new(Payment::ReceivedByNodeId).string())
                    .col(ColumnDef::new(Payment::CreatedByNodeId).string().not_null())
                    .col(ColumnDef::new(Payment::PaymentHash).string().not_null())
                    .col(ColumnDef::new(Payment::Preimage).string())
                    .col(ColumnDef::new(Payment::Secret).string())
                    .col(ColumnDef::new(Payment::Status).string().not_null())
                    .col(ColumnDef::new(Payment::Origin).string().not_null())
                    .col(ColumnDef::new(Payment::AmtMsat).big_integer())
                    .col(ColumnDef::new(Payment::FeePaidMsat).big_integer())
                    .col(ColumnDef::new(Payment::Label).string())
                    .col(ColumnDef::new(Payment::Invoice).text())
                    .to_owned(),
            )
            .await;

        manager
            .create_index(
                Index::create()
                    .table(Payment::Table)
                    .name("idx-nodeid-paymenthash")
                    .col(Payment::NodeId)
                    .col(Payment::PaymentHash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut stmt = Table::drop();
        stmt.table(Payment::Table);
        manager.drop_table(stmt).await
    }
}

#[derive(Iden)]
enum Payment {
    Table,
    Id,
    NodeId,
    ReceivedByNodeId,
    CreatedByNodeId,
    PaymentHash,
    Preimage,
    Secret,
    Status,
    Origin,
    AmtMsat,
    FeePaidMsat,
    CreatedAt,
    UpdatedAt,
    Label,
    Invoice,
}
