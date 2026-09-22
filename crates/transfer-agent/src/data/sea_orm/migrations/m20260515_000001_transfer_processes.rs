/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260515_000001_transfer_processes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferProcesses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransferProcesses::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::TenantId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferProcesses::Role).string().not_null())
                    .col(
                        ColumnDef::new(TransferProcesses::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::Version)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::Protocol)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::ProtocolState)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::StateMetadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::AgreementId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::PeerParticipantId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::Correlation)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::Properties)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::ErrorDetails)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::LastInboundEnvelope)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TransferProcesses::LastOutboundEnvelope)
                            .json_binary()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TransferProcesses::Table)
                    .col(TransferProcesses::TenantId)
                    .name("idx_tp_tenant_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TransferProcesses::Table)
                    .col(TransferProcesses::CreatedAt)
                    .name("idx_tp_created_at")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransferProcesses::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TransferProcesses {
    Table,
    Id,
    TenantId,
    Role,
    CreatedAt,
    UpdatedAt,
    Version,
    Protocol,
    ProtocolState,
    StateMetadata,
    AgreementId,
    PeerParticipantId,
    Correlation,
    Properties,
    ErrorDetails,
    LastInboundEnvelope,
    LastOutboundEnvelope,
}
