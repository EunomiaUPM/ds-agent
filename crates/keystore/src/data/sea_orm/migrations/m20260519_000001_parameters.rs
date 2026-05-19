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
        "m20260519_000001_keystore_parameters"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(KeystoreParameters::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(KeystoreParameters::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(KeystoreParameters::Value)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeystoreParameters::Version)
                            .big_integer()
                            .not_null()
                            .default(1i64),
                    )
                    .col(ColumnDef::new(KeystoreParameters::Description).string())
                    .col(
                        ColumnDef::new(KeystoreParameters::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeystoreParameters::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeystoreParameters::CreatedBy)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(KeystoreParameters::UpdatedBy)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(KeystoreParameters::DeletedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KeystoreParameters::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum KeystoreParameters {
    Table,
    Key,
    Value,
    Version,
    Description,
    CreatedAt,
    UpdatedAt,
    CreatedBy,
    UpdatedBy,
    DeletedAt,
}
