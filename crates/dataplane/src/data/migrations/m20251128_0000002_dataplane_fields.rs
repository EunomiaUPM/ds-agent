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
        "m20251128_0000002_dataplane_fields"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DataplaneFields::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DataplaneFields::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DataplaneFields::Key).string().not_null())
                    .col(ColumnDef::new(DataplaneFields::Value).string())
                    .col(
                        ColumnDef::new(DataplaneFields::DataplaneProcessId)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_dataplane_fields_dataplane_process")
                            .from(DataplaneFields::Table, DataplaneFields::DataplaneProcessId)
                            .to(DataplaneTransfers::Table, DataplaneTransfers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DataplaneFields::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum DataplaneFields {
    Table,
    Id,
    Key,
    Value,
    DataplaneProcessId,
}
