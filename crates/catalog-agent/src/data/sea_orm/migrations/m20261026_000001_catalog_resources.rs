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

//! Every DCAT resource in one table: index columns for what queries and keys need, the RDF
//! description whole in `jsonb`.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261026_000001_catalog_resources"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogResources::Table)
                    .col(ColumnDef::new(CatalogResources::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(CatalogResources::Kind).string_len(32).not_null())
                    .col(ColumnDef::new(CatalogResources::UserId).string().not_null())
                    .col(ColumnDef::new(CatalogResources::UserRole).string_len(255).not_null())
                    .col(ColumnDef::new(CatalogResources::Visibility).string_len(16).not_null())
                    .col(ColumnDef::new(CatalogResources::CatalogId).string().null())
                    .col(ColumnDef::new(CatalogResources::DatasetId).string().null())
                    .col(
                        ColumnDef::new(CatalogResources::IsMain)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(CatalogResources::Title).text().null())
                    .col(
                        ColumnDef::new(CatalogResources::Issued)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CatalogResources::Modified).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(CatalogResources::Version)
                            .big_integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(CatalogResources::Description).json_binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_resources_catalog")
                            .from(CatalogResources::Table, CatalogResources::CatalogId)
                            .to(CatalogResources::Table, CatalogResources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_resources_dataset")
                            .from(CatalogResources::Table, CatalogResources::DatasetId)
                            .to(CatalogResources::Table, CatalogResources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        let db = manager.get_connection();
        // One main catalog and one main data service per connector, enforced by the engine.
        db.execute_unprepared(
            "CREATE UNIQUE INDEX ux_catalog_resources_main \
             ON catalog_resources (kind) WHERE is_main",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX ix_catalog_resources_listing \
             ON catalog_resources (kind, issued DESC, id DESC)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX ix_catalog_resources_catalog ON catalog_resources (catalog_id)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX ix_catalog_resources_dataset ON catalog_resources (dataset_id)",
        )
        .await?;
        // Containment queries over the description (keyword, theme, format…).
        db.execute_unprepared(
            "CREATE INDEX ix_catalog_resources_description \
             ON catalog_resources USING GIN (description jsonb_path_ops)",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogResources::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CatalogResources {
    Table,
    Id,
    Kind,
    UserId,
    UserRole,
    Visibility,
    CatalogId,
    DatasetId,
    IsMain,
    Title,
    Issued,
    Modified,
    Version,
    Description,
}
