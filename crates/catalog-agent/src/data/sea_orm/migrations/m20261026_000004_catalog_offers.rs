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

//! ODRL offers, each on one resource through a real foreign key, and on the template version it
//! was made from, if any.

use sea_orm_migration::prelude::*;

use crate::data::sea_orm::migrations::m20261026_000001_catalog_resources::CatalogResources;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261026_000004_catalog_offers"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogOffers::Table)
                    .col(ColumnDef::new(CatalogOffers::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(CatalogOffers::UserId).string().not_null())
                    .col(ColumnDef::new(CatalogOffers::UserRole).string_len(255).not_null())
                    .col(ColumnDef::new(CatalogOffers::Visibility).string_len(16).not_null())
                    .col(
                        ColumnDef::new(CatalogOffers::OdrlOffer)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(CatalogOffers::Entity).string().not_null())
                    .col(ColumnDef::new(CatalogOffers::EntityType).string().not_null())
                    .col(
                        ColumnDef::new(CatalogOffers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CatalogOffers::SourceTemplateId).string().null())
                    .col(ColumnDef::new(CatalogOffers::SourceTemplateVersion).string().null())
                    .col(
                        ColumnDef::new(CatalogOffers::InstantiationParameters)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(CatalogOffers::Description).text().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_offers_entity")
                            .from(CatalogOffers::Table, CatalogOffers::Entity)
                            .to(CatalogResources::Table, CatalogResources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        // Templates are unique by id and version, whoever owns them (and an offer may come from a
        // template of someone else it sees, like the connector's library).
        db.execute_unprepared(
            "ALTER TABLE catalog_offers \
             ADD CONSTRAINT fk_catalog_offers_template \
             FOREIGN KEY (source_template_id, source_template_version) \
             REFERENCES catalog_policy_templates (id, version) \
             ON DELETE SET NULL (source_template_id, source_template_version) \
             ON UPDATE CASCADE",
        )
        .await?;
        db.execute_unprepared("CREATE INDEX ix_catalog_offers_entity ON catalog_offers (entity)")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogOffers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CatalogOffers {
    Table,
    Id,
    UserId,
    UserRole,
    Visibility,
    OdrlOffer,
    Entity,
    EntityType,
    CreatedAt,
    SourceTemplateId,
    SourceTemplateVersion,
    InstantiationParameters,
    Description,
}
