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
        "m20251118_000003_transfer_process_identifiers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferAgentIdentifiers::Table)
                    .col(
                        ColumnDef::new(TransferAgentIdentifiers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransferAgentIdentifiers::TransferAgentProcessId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferAgentIdentifiers::IdKey).string().not_null())
                    .col(ColumnDef::new(TransferAgentIdentifiers::IdValue).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transfer_identifiers-process_id")
                            .from(
                                TransferAgentIdentifiers::Table,
                                TransferAgentIdentifiers::TransferAgentProcessId,
                            )
                            .to(TransferAgentProcess::Table, TransferAgentProcess::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(TransferAgentIdentifiers::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum TransferAgentIdentifiers {
    Table,
    Id,
    TransferAgentProcessId,
    IdKey,
    IdValue,
}

#[derive(Iden)]
pub enum TransferAgentProcess {
    Table,
    Id,
}
