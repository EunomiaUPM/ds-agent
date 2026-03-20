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
                    .col(ColumnDef::new(CatalogODRLOffers::Id).string().not_null().primary_key())
                    .col(
                        ColumnDef::new(CatalogODRLOffers::ODRLOffer)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(CatalogODRLOffers::Entity).string().not_null())
                    .col(ColumnDef::new(CatalogODRLOffers::EntityType).string().not_null())
                    .col(
                        ColumnDef::new(CatalogODRLOffers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CatalogODRLOffers::SourceTemplateId).string().null())
                    .col(ColumnDef::new(CatalogODRLOffers::SourceTemplateVersion).string().null())
                    .col(
                        ColumnDef::new(CatalogODRLOffers::InstantiationParameters)
                            .json_binary()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_odrl_offers_template_source")
                            .from(
                                CatalogODRLOffers::Table,
                                (
                                    CatalogODRLOffers::SourceTemplateId,
                                    CatalogODRLOffers::SourceTemplateVersion,
                                ),
                            )
                            .to(
                                PolicyTemplates::Table,
                                (PolicyTemplates::Id, PolicyTemplates::Version),
                            )
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(CatalogODRLOffers::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum CatalogODRLOffers {
    Table,
    Id,
    ODRLOffer,
    Entity,
    EntityType,
    CreatedAt,
    SourceTemplateId,
    SourceTemplateVersion,
    InstantiationParameters,
}

#[derive(Iden)]
pub enum PolicyTemplates {
    Table,
    Id,
    Version,
}
