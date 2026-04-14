/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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
        "m20251118_000001_negotiation_process"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NegotiationAgentProcess::Table)
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::State)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NegotiationAgentProcess::StateAttribute).string())
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::AssociatedAgentPeer)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::Protocol)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NegotiationAgentProcess::CallbackAddress).string())
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::Role)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::Properties)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NegotiationAgentProcess::ErrorDetails).json_binary())
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentProcess::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-negotiation_agent_process-properties_gin")
                    .table(NegotiationAgentProcess::Table)
                    .col(NegotiationAgentProcess::Properties)
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
                    .name("idx-negotiation_agent_process-properties_gin")
                    .table(NegotiationAgentProcess::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(NegotiationAgentProcess::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum NegotiationAgentProcess {
    Table,
    Id,
    State,
    StateAttribute,
    AssociatedAgentPeer,
    Protocol,
    CallbackAddress,
    Role,
    Properties,
    ErrorDetails,
    CreatedAt,
    UpdatedAt,
}
