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
        "m20241111_000002_distribution"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CatalogDistributions::Table)
                    .col(ColumnDef::new(CatalogDistributions::Id).string().not_null().primary_key())
                    .col(
                        ColumnDef::new(CatalogDistributions::DctIssued)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CatalogDistributions::DctModified)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(CatalogDistributions::DctTitle).string())
                    .col(ColumnDef::new(CatalogDistributions::DctDescription).string())
                    .col(ColumnDef::new(CatalogDistributions::DctFormat).string())
                    .col(
                        ColumnDef::new(CatalogDistributions::DcatAccessService).string().not_null(),
                    )
                    .col(ColumnDef::new(CatalogDistributions::DatasetId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_distribution_dataset")
                            .from(CatalogDistributions::Table, CatalogDistributions::DatasetId)
                            .to(CatalogDatasets::Table, CatalogDatasets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(CatalogDistributions::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum CatalogDistributions {
    Table,
    Id,
    DctIssued,
    DctModified,
    DctTitle,
    DctDescription,
    DctFormat,
    DcatAccessService,
    DatasetId,
}

#[derive(Iden)]
pub enum CatalogDatasets {
    Table,
    Id,
}
