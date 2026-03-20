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
        "m20251111_000001_connector_template"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create Table
        manager
            .create_table(
                Table::create()
                    .table(ConnectorTemplates::Table)
                    .col(ColumnDef::new(ConnectorTemplates::Name).string().not_null())
                    .col(ColumnDef::new(ConnectorTemplates::Version).string().not_null())
                    .col(ColumnDef::new(ConnectorTemplates::Author).string().not_null())
                    .col(
                        ColumnDef::new(ConnectorTemplates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ConnectorTemplates::Spec).json_binary().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_connector_template_name_version")
                    .table(ConnectorTemplates::Table)
                    .col(ConnectorTemplates::Name)
                    .col(ConnectorTemplates::Version)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(ConnectorTemplates::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum ConnectorTemplates {
    Table,
    Name,
    Version,
    Author,
    CreatedAt,
    Spec,
}
