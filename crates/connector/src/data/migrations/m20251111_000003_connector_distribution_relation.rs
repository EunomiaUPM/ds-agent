/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
        "m20251111_000003_connector_distribution_relation"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ConnectorDistroRelations::Table)
                    .col(
                        ColumnDef::new(ConnectorDistroRelations::DistributionId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ConnectorDistroRelations::ConnectorInstanceId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_instance_template")
                            .from(
                                ConnectorDistroRelations::Table,
                                ConnectorDistroRelations::ConnectorInstanceId,
                            )
                            .to(ConnectorInstances::Table, ConnectorInstances::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ConnectorDistroRelations::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum ConnectorDistroRelations {
    Table,
    DistributionId,
    ConnectorInstanceId,
}

#[derive(Iden)]
pub enum ConnectorInstances {
    Table,
    Id,
}
