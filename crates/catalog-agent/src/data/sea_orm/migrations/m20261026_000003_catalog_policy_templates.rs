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

//! Policy templates, one row per template version.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261026_000003_catalog_policy_templates"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogPolicyTemplates::Table)
                    .col(ColumnDef::new(CatalogPolicyTemplates::Id).string().not_null())
                    .col(ColumnDef::new(CatalogPolicyTemplates::Version).string().not_null())
                    .col(ColumnDef::new(CatalogPolicyTemplates::UserId).string().not_null())
                    .col(
                        ColumnDef::new(CatalogPolicyTemplates::UserRole)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogPolicyTemplates::Visibility)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogPolicyTemplates::Date)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CatalogPolicyTemplates::Author).string().not_null())
                    .col(ColumnDef::new(CatalogPolicyTemplates::Title).json_binary())
                    .col(ColumnDef::new(CatalogPolicyTemplates::Description).json_binary())
                    .col(ColumnDef::new(CatalogPolicyTemplates::Content).json_binary().not_null())
                    .col(
                        ColumnDef::new(CatalogPolicyTemplates::Parameters)
                            .json_binary()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_catalog_policy_templates")
                            .col(CatalogPolicyTemplates::Id)
                            .col(CatalogPolicyTemplates::Version),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogPolicyTemplates::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CatalogPolicyTemplates {
    Table,
    Id,
    Version,
    UserId,
    UserRole,
    Visibility,
    Date,
    Author,
    Title,
    Description,
    Content,
    Parameters,
}
