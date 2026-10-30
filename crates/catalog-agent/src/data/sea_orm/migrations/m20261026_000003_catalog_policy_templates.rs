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

//! Policy templates, one row per template version, and which versions each one references.

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
                    .col(
                        ColumnDef::new(CatalogPolicyTemplates::Kind)
                            .string_len(16)
                            .not_null()
                            .default("policy"),
                    )
                    .col(
                        ColumnDef::new(CatalogPolicyTemplates::ControlPoints)
                            .json_binary()
                            .not_null()
                            .default("[]"),
                    )
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
            .await?;
        // A version another one references cannot go; removing the referencing one frees it.
        manager
            .create_table(
                Table::create()
                    .table(CatalogPolicyTemplateRefs::Table)
                    .col(ColumnDef::new(CatalogPolicyTemplateRefs::TemplateId).string().not_null())
                    .col(ColumnDef::new(CatalogPolicyTemplateRefs::TemplateVersion).string().not_null())
                    .col(ColumnDef::new(CatalogPolicyTemplateRefs::RefId).string().not_null())
                    .col(ColumnDef::new(CatalogPolicyTemplateRefs::RefVersion).string().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_catalog_policy_template_refs")
                            .col(CatalogPolicyTemplateRefs::TemplateId)
                            .col(CatalogPolicyTemplateRefs::TemplateVersion)
                            .col(CatalogPolicyTemplateRefs::RefId)
                            .col(CatalogPolicyTemplateRefs::RefVersion),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_policy_template_refs_template")
                            .from_tbl(CatalogPolicyTemplateRefs::Table)
                            .from_col(CatalogPolicyTemplateRefs::TemplateId)
                            .from_col(CatalogPolicyTemplateRefs::TemplateVersion)
                            .to_tbl(CatalogPolicyTemplates::Table)
                            .to_col(CatalogPolicyTemplates::Id)
                            .to_col(CatalogPolicyTemplates::Version)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_policy_template_refs_ref")
                            .from_tbl(CatalogPolicyTemplateRefs::Table)
                            .from_col(CatalogPolicyTemplateRefs::RefId)
                            .from_col(CatalogPolicyTemplateRefs::RefVersion)
                            .to_tbl(CatalogPolicyTemplates::Table)
                            .to_col(CatalogPolicyTemplates::Id)
                            .to_col(CatalogPolicyTemplates::Version)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX ix_catalog_policy_template_refs_ref \
                 ON catalog_policy_template_refs (ref_id, ref_version)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogPolicyTemplateRefs::Table).to_owned())
            .await?;
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
    Kind,
    ControlPoints,
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

#[derive(Iden)]
pub enum CatalogPolicyTemplateRefs {
    Table,
    TemplateId,
    TemplateVersion,
    RefId,
    RefVersion,
}
