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
        "m20251118_000002_negotiation_messages"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NegotiationAgentMessages::Table)
                    .col(
                        ColumnDef::new(NegotiationAgentMessages::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentMessages::NegotiationAgentProcessId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentMessages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NegotiationAgentMessages::Direction).string().not_null())
                    .col(ColumnDef::new(NegotiationAgentMessages::Protocol).string().not_null())
                    .col(ColumnDef::new(NegotiationAgentMessages::MessageType).string().not_null())
                    .col(
                        ColumnDef::new(NegotiationAgentMessages::StateTransitionFrom)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentMessages::StateTransitionTo)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NegotiationAgentMessages::Payload).json_binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-negotiation_messages-process_id")
                            .from(
                                NegotiationAgentMessages::Table,
                                NegotiationAgentMessages::NegotiationAgentProcessId,
                            )
                            .to(NegotiationAgentProcess::Table, NegotiationAgentProcess::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(NegotiationAgentMessages::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum NegotiationAgentMessages {
    Table,
    Id,
    NegotiationAgentProcessId,
    CreatedAt,
    Direction,
    Protocol,
    MessageType,
    StateTransitionFrom,
    StateTransitionTo,
    Payload,
}

#[derive(Iden)]
pub enum NegotiationAgentProcess {
    Table,
    Id,
}
