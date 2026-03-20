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
        "m20251118_000002_transfer_messages"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferAgentMessages::Table)
                    .col(
                        ColumnDef::new(TransferAgentMessages::Id).string().not_null().primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransferAgentMessages::TransferAgentProcessId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferAgentMessages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferAgentMessages::Direction).string().not_null())
                    .col(ColumnDef::new(TransferAgentMessages::Protocol).string().not_null())
                    .col(ColumnDef::new(TransferAgentMessages::MessageType).string().not_null())
                    .col(
                        ColumnDef::new(TransferAgentMessages::StateTransitionFrom)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferAgentMessages::StateTransitionTo)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferAgentMessages::Payload).json_binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transfer_messages-process_id")
                            .from(
                                TransferAgentMessages::Table,
                                TransferAgentMessages::TransferAgentProcessId,
                            )
                            .to(TransferAgentProcess::Table, TransferAgentProcess::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(TransferAgentMessages::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum TransferAgentMessages {
    Table,
    Id,
    TransferAgentProcessId,
    CreatedAt,
    Direction,
    Protocol,
    MessageType,
    StateTransitionFrom,
    StateTransitionTo,
    Payload,
}

#[derive(Iden)]
pub enum TransferAgentProcess {
    Table,
    Id,
}
