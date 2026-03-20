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
        "m20241111_000005_policy_templates"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PolicyTemplates::Table)
                    .col(ColumnDef::new(PolicyTemplates::Id).string().not_null())
                    .col(ColumnDef::new(PolicyTemplates::Version).string().not_null())
                    .col(
                        ColumnDef::new(PolicyTemplates::Date).timestamp_with_time_zone().not_null(),
                    )
                    .col(ColumnDef::new(PolicyTemplates::Author).string().not_null())
                    .col(ColumnDef::new(PolicyTemplates::Title).json_binary())
                    .col(ColumnDef::new(PolicyTemplates::Description).json_binary())
                    .col(ColumnDef::new(PolicyTemplates::Content).json_binary().not_null())
                    .col(ColumnDef::new(PolicyTemplates::Parameters).json_binary().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_policy_templates")
                            .col(PolicyTemplates::Id)
                            .col(PolicyTemplates::Version),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(PolicyTemplates::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum PolicyTemplates {
    Table,
    Id,
    Version,
    Date,
    Author,
    Title,
    Description,
    Content,
    Parameters,
}
