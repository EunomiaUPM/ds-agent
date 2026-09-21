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
        "m20241111_000006_policies"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogODRLOffers::Table)
                    .col(
                        ColumnDef::new(CatalogODRLOffers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::TenantId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::ODRLOffer)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::Entity)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::EntityType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::SourceTemplateId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::SourceTemplateVersion)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(CatalogODRLOffers::InstantiationParameters)
                            .json_binary()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // sea-query cannot express a column-subset SET NULL, needed because tenant_id is NOT NULL.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE catalog_odrl_offers \
                 ADD CONSTRAINT fk_odrl_offers_template_source \
                 FOREIGN KEY (tenant_id, source_template_id, source_template_version) \
                 REFERENCES policy_templates (tenant_id, id, version) \
                 ON DELETE SET NULL (source_template_id, source_template_version) \
                 ON UPDATE CASCADE",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogODRLOffers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CatalogODRLOffers {
    Table,
    Id,
    TenantId,
    ODRLOffer,
    Entity,
    EntityType,
    CreatedAt,
    SourceTemplateId,
    SourceTemplateVersion,
    InstantiationParameters,
}
