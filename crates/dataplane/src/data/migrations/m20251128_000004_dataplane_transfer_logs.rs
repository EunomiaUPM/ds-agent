/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
                    .col(
                        ColumnDef::new(DataplaneTransferLogs::Id).string().not_null().primary_key(),
                    )
                    .col(
                        ColumnDef::new(DataplaneTransferLogs::DataplaneProcessId)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_logs_transfer_id")
                            .from(
                                DataplaneTransferLogs::Table,
                                DataplaneTransferLogs::DataplaneProcessId,
                            )
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
    DataplaneProcessId,
    PreviousState,
    NewState,
    Trigger,
    Reason,
    CreatedAt,
}
