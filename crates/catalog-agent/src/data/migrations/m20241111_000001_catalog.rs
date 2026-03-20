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
        "m20241111_000001_catalog"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogCatalogs::Table)
                    .col(ColumnDef::new(CatalogCatalogs::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(CatalogCatalogs::FoafHomePage).string())
                    .col(ColumnDef::new(CatalogCatalogs::DctConformsTo).string())
                    .col(ColumnDef::new(CatalogCatalogs::DctCreator).string())
                    .col(ColumnDef::new(CatalogCatalogs::DctIdentifier).string())
                    .col(
                        ColumnDef::new(CatalogCatalogs::DctIssued)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CatalogCatalogs::DctModified).timestamp_with_time_zone())
                    .col(ColumnDef::new(CatalogCatalogs::DctTitle).string())
                    .col(ColumnDef::new(CatalogCatalogs::DspaceParticipantId).string())
                    .col(
                        ColumnDef::new(CatalogCatalogs::DspaceMainCatalog)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(CatalogCatalogs::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum CatalogCatalogs {
    Table,
    Id,
    FoafHomePage,
    DctConformsTo,
    DctCreator,
    DctIdentifier,
    DctIssued,
    DctModified,
    DctTitle,
    DspaceParticipantId,
    DspaceMainCatalog,
}
