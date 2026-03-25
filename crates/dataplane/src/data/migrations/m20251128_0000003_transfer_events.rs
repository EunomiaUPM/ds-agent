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

use crate::data::migrations::m20251128_0000001_dataplane_transfers::DataplaneTransfers;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20251128_0000003_transfer_events"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransferEvents::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransferEvents::TransferId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferEvents::Level).string().not_null())
                    .col(
                        ColumnDef::new(TransferEvents::Component)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferEvents::Message).text().not_null())
                    .col(ColumnDef::new(TransferEvents::Data).json_binary().null())
                    .col(
                        ColumnDef::new(TransferEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transfer_events_dataplane_transfer")
                            .from(TransferEvents::Table, TransferEvents::TransferId)
                            .to(DataplaneTransfers::Table, DataplaneTransfers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransferEvents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TransferEvents {
    Table,
    Id,
    TransferId,
    Level,
    Component,
    Message,
    Data,
    CreatedAt,
}
