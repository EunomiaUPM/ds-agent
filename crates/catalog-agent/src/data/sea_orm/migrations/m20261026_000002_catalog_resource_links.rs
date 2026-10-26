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

//! Links between resources (`dcat:accessService`, `dct:publisher`…) as real foreign keys: a
//! resource cannot point to one that is not there, nor can a linked one be removed.

use sea_orm_migration::prelude::*;

use crate::data::sea_orm::migrations::m20261026_000001_catalog_resources::CatalogResources;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261026_000002_catalog_resource_links"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogResourceLinks::Table)
                    .col(ColumnDef::new(CatalogResourceLinks::Subject).string().not_null())
                    .col(ColumnDef::new(CatalogResourceLinks::Predicate).string().not_null())
                    .col(ColumnDef::new(CatalogResourceLinks::Object).string().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_catalog_resource_links")
                            .col(CatalogResourceLinks::Subject)
                            .col(CatalogResourceLinks::Predicate)
                            .col(CatalogResourceLinks::Object),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_resource_links_subject")
                            .from(CatalogResourceLinks::Table, CatalogResourceLinks::Subject)
                            .to(CatalogResources::Table, CatalogResources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_catalog_resource_links_object")
                            .from(CatalogResourceLinks::Table, CatalogResourceLinks::Object)
                            .to(CatalogResources::Table, CatalogResources::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;
        // Who points to a resource: what blocks its removal and what an inverse link reads.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX ix_catalog_resource_links_object \
                 ON catalog_resource_links (object, predicate)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CatalogResourceLinks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CatalogResourceLinks {
    Table,
    Subject,
    Predicate,
    Object,
}
