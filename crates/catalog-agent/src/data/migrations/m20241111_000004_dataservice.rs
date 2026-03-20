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
        "m20241111_000004_dataservice"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogDataServices::Table)
                    .col(ColumnDef::new(CatalogDataServices::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(CatalogDataServices::DcatEndpointDescription).string())
                    .col(ColumnDef::new(CatalogDataServices::DcatEndpointURL).string().not_null())
                    .col(ColumnDef::new(CatalogDataServices::DctConformsTo).string())
                    .col(ColumnDef::new(CatalogDataServices::DctCreator).string())
                    .col(ColumnDef::new(CatalogDataServices::DctIdentifier).string())
                    .col(
                        ColumnDef::new(CatalogDataServices::DctIssued)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogDataServices::DctModified).timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(CatalogDataServices::DctTitle).string())
                    .col(ColumnDef::new(CatalogDataServices::DctDescription).string())
                    .col(ColumnDef::new(CatalogDataServices::CatalogId).string().not_null())
                    .col(
                        ColumnDef::new(CatalogDataServices::DspaceMainDataService)
                            .boolean()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_dataservice_catalog")
                            .from(CatalogDataServices::Table, CatalogDataServices::CatalogId)
                            .to(CatalogCatalogs::Table, CatalogCatalogs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(CatalogDataServices::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum CatalogDataServices {
    Table,
    Id,
    DcatEndpointDescription,
    DcatEndpointURL,
    DctConformsTo,
    DctCreator,
    DctIdentifier,
    DctIssued,
    DctModified,
    DctTitle,
    DctDescription,
    CatalogId,
    DspaceMainDataService,
}

#[derive(Iden)]
pub enum CatalogCatalogs {
    Table,
    Id,
}
