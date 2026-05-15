use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str { "m20260515_000002_transfer_messages" }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferMessages::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TransferMessages::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(TransferMessages::TransferProcessId).string().not_null())
                    .col(ColumnDef::new(TransferMessages::TenantId).string().not_null())
                    .col(ColumnDef::new(TransferMessages::Direction).string().not_null())
                    .col(ColumnDef::new(TransferMessages::Protocol).string().not_null())
                    .col(ColumnDef::new(TransferMessages::MessageType).string().not_null())
                    .col(ColumnDef::new(TransferMessages::ProtocolVersion).string().not_null().default("1.0"))
                    .col(ColumnDef::new(TransferMessages::Envelope).json_binary().not_null())
                    .col(ColumnDef::new(TransferMessages::OccurredAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TransferMessages::CorrelationId).string().null())
                    .col(ColumnDef::new(TransferMessages::RequestId).string().not_null())
                    .col(ColumnDef::new(TransferMessages::PeerParticipantId).string().not_null())
                    .col(ColumnDef::new(TransferMessages::ProcessingResult).json_binary().not_null())
                    .col(ColumnDef::new(TransferMessages::StateTransitionTo).string().null())
                    .to_owned(),
            )
            .await?;

        manager.create_index(
            Index::create()
                .table(TransferMessages::Table)
                .col(TransferMessages::TransferProcessId)
                .name("idx_tm_transfer_process_id")
                .to_owned(),
        ).await?;

        manager.create_index(
            Index::create()
                .table(TransferMessages::Table)
                .col(TransferMessages::TenantId)
                .name("idx_tm_tenant_id")
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(TransferMessages::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum TransferMessages {
    Table,
    Id,
    TransferProcessId,
    TenantId,
    Direction,
    Protocol,
    MessageType,
    ProtocolVersion,
    Envelope,
    OccurredAt,
    CorrelationId,
    RequestId,
    PeerParticipantId,
    ProcessingResult,
    StateTransitionTo,
}
