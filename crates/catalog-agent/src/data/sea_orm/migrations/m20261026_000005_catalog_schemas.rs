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

//! Hosted JSON Schemas, one immutable row per version.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261026_000005_catalog_schemas"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogSchemas::Table)
                    .col(ColumnDef::new(CatalogSchemas::Id).string().not_null())
                    .col(ColumnDef::new(CatalogSchemas::Version).integer().not_null())
                    .col(ColumnDef::new(CatalogSchemas::UserId).string().not_null())
                    .col(
                        ColumnDef::new(CatalogSchemas::UserRole)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogSchemas::Visibility)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogSchemas::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CatalogSchemas::Title).string().not_null())
                    .col(ColumnDef::new(CatalogSchemas::Description).text())
                    .col(
                        ColumnDef::new(CatalogSchemas::Content)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogSchemas::Sha256)
                            .string_len(64)
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_catalog_schemas")
                            .col(CatalogSchemas::Id)
                            .col(CatalogSchemas::Version),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogSchemas::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CatalogSchemas {
    Table,
    Id,
    Version,
    UserId,
    UserRole,
    Visibility,
    CreatedAt,
    Title,
    Description,
    Content,
    Sha256,
}
