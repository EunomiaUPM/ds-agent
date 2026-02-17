use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251128_0000001_dataplane_transfers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DataplaneTransfers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DataplaneTransfers::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(DataplaneTransfers::TransferProcessId)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DataplaneTransfers::Role).string().not_null())
                    .col(ColumnDef::new(DataplaneTransfers::InteractionMode).string().not_null())
                    .col(ColumnDef::new(DataplaneTransfers::State).string().not_null())
                    .col(ColumnDef::new(DataplaneTransfers::ConnectorInstanceId).uuid().null())
                    .col(ColumnDef::new(DataplaneTransfers::IngressConfig).json_binary().not_null())
                    .col(ColumnDef::new(DataplaneTransfers::EgressConfig).json_binary().not_null())
                    .col(ColumnDef::new(DataplaneTransfers::FlowControl).json_binary().null())
                    .col(
                        ColumnDef::new(DataplaneTransfers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DataplaneTransfers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(DataplaneTransfers::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum DataplaneTransfers {
    Table,
    Id,
    TransferProcessId,
    Role,
    InteractionMode,
    State,
    ConnectorInstanceId,
    IngressConfig,
    EgressConfig,
    FlowControl,
    CreatedAt,
    UpdatedAt,
}
