use crate::data::migrations::m20251128_0000001_dataplane_transfers::DataplaneTransfers;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251128_000004_dataplane_transfer_logs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create dataplane_transfer_logs table
        manager
            .create_table(
                Table::create()
                    .table(DataplaneTransferLogs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DataplaneTransferLogs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(DataplaneTransferLogs::TransferId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_logs_transfer_id")
                            .from(DataplaneTransferLogs::Table, DataplaneTransferLogs::TransferId)
                            .to(DataplaneTransfers::Table, DataplaneTransfers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(DataplaneTransferLogs::PreviousState).string().null())
                    .col(ColumnDef::new(DataplaneTransferLogs::NewState).string().not_null())
                    .col(ColumnDef::new(DataplaneTransferLogs::Trigger).string().not_null())
                    .col(ColumnDef::new(DataplaneTransferLogs::Reason).text().null())
                    .col(
                        ColumnDef::new(DataplaneTransferLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DataplaneTransferLogs::Table).to_owned()).await?;

        Ok(())
    }
}

#[derive(Iden)]
enum DataplaneTransferLogs {
    Table,
    Id,
    TransferId,
    PreviousState,
    NewState,
    Trigger,
    Reason,
    CreatedAt,
}
