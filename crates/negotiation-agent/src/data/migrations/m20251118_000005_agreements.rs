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
        "m20251118_000005_agreements"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NegotiationAgentAgreements::Table)
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::NegotiationAgentProcessId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::NegotiationAgentMessageId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::ConsumerParticipantId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::ProviderParticipantId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::AgreementContent)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NegotiationAgentAgreements::Target).string().not_null())
                    .col(ColumnDef::new(NegotiationAgentAgreements::State).string().not_null())
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NegotiationAgentAgreements::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-negotiation_agreement-process_id")
                            .from(
                                NegotiationAgentAgreements::Table,
                                NegotiationAgentAgreements::NegotiationAgentProcessId,
                            )
                            .to(NegotiationAgentProcess::Table, NegotiationAgentProcess::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-negotiation_agreement-message_id")
                            .from(
                                NegotiationAgentAgreements::Table,
                                NegotiationAgentAgreements::NegotiationAgentMessageId,
                            )
                            .to(NegotiationAgentMessages::Table, NegotiationAgentMessages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(NegotiationAgentAgreements::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum NegotiationAgentAgreements {
    Table,
    Id,
    NegotiationAgentProcessId,
    NegotiationAgentMessageId,
    ConsumerParticipantId,
    ProviderParticipantId,
    AgreementContent,
    Target,
    State,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum NegotiationAgentProcess {
    Table,
    Id,
}

#[derive(Iden)]
pub enum NegotiationAgentMessages {
    Table,
    Id,
}
