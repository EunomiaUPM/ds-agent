/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251118_000001_transfer_processes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferAgentProcess::Table)
                    .col(ColumnDef::new(TransferAgentProcess::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(TransferAgentProcess::State).string().not_null())
                    .col(ColumnDef::new(TransferAgentProcess::StateAttribute).string())
                    .col(
                        ColumnDef::new(TransferAgentProcess::AssociatedAgentPeer)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferAgentProcess::Protocol).string().not_null())
                    .col(
                        ColumnDef::new(TransferAgentProcess::TransferDirection).string().not_null(),
                    )
                    .col(ColumnDef::new(TransferAgentProcess::AgreementId).string().not_null())
                    .col(ColumnDef::new(TransferAgentProcess::CallbackAddress).string())
                    .col(ColumnDef::new(TransferAgentProcess::Role).string().not_null())
                    .col(ColumnDef::new(TransferAgentProcess::Properties).json_binary().not_null())
                    .col(ColumnDef::new(TransferAgentProcess::ErrorDetails).json_binary())
                    .col(
                        ColumnDef::new(TransferAgentProcess::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferAgentProcess::UpdatedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-transfer_agent_process-properties_gin")
                    .table(TransferAgentProcess::Table)
                    .col(TransferAgentProcess::Properties)
                    .index_type(IndexType::Custom(DynIden::new("GIN")))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-transfer_agent_process-properties_gin") // Sin '?' aquí
                    .table(TransferAgentProcess::Table) // Es buena práctica especificar la tabla también
                    .to_owned(),
            )
            .await?;
        manager.drop_table(Table::drop().table(TransferAgentProcess::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum TransferAgentProcess {
    Table,
    Id,
    State,
    StateAttribute,
    AssociatedAgentPeer,
    Protocol,
    TransferDirection,
    AgreementId,
    CallbackAddress,
    Role,
    Properties,
    ErrorDetails,
    CreatedAt,
    UpdatedAt,
}
