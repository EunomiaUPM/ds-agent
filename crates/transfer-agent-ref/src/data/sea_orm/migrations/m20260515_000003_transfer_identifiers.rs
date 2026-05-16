use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260515_000003_transfer_identifiers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferIdentifiers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransferIdentifiers::TransferProcessId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferIdentifiers::Key).string().not_null())
                    .col(ColumnDef::new(TransferIdentifiers::Value).string().null())
                    .primary_key(
                        Index::create()
                            .col(TransferIdentifiers::TransferProcessId)
                            .col(TransferIdentifiers::Key),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransferIdentifiers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TransferIdentifiers {
    Table,
    TransferProcessId,
    Key,
    Value,
}
