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

use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241123_0000001_subscriptions"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .col(
                        ColumnDef::new(Subscriptions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subscriptions::TenantId).string().not_null())
                    .col(
                        ColumnDef::new(Subscriptions::CallbackAddress)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::TopicPattern)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Subscriptions::Secret).string())
                    .col(ColumnDef::new(Subscriptions::Headers).json())
                    .col(ColumnDef::new(Subscriptions::RetryLimit).integer())
                    .col(
                        ColumnDef::new(Subscriptions::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Subscriptions::UpdatedAt).date_time())
                    .col(ColumnDef::new(Subscriptions::ExpirationTime).date_time())
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Subscriptions {
    Table,
    Id,
    TenantId,
    CallbackAddress,
    TopicPattern,
    Secret,
    Headers,
    RetryLimit,
    Active,
    CreatedAt,
    UpdatedAt,
    ExpirationTime,
}
