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

use crate::data::migrations::m20241123_0000001_subscriptions::Subscriptions;
use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241123_0000002_notifications"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notifications::Table)
                    .col(ColumnDef::new(Notifications::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Notifications::Timestamp).date_time().not_null())
                    .col(ColumnDef::new(Notifications::Category).string().not_null())
                    .col(ColumnDef::new(Notifications::Subcategory).string().not_null())
                    .col(ColumnDef::new(Notifications::MessageType).string().not_null())
                    .col(ColumnDef::new(Notifications::MessageContent).json().not_null())
                    .col(ColumnDef::new(Notifications::MessageOperation).string().not_null())
                    .col(ColumnDef::new(Notifications::Status).string().not_null())
                    .col(ColumnDef::new(Notifications::SubscriptionId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notifications_subscriptions")
                            .from(Notifications::Table, Notifications::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Notifications::Table).to_owned()).await
    }
}

#[derive(Iden)]
pub enum Notifications {
    Table,
    Id,
    Timestamp,
    Category,
    Subcategory,
    MessageType,
    MessageContent,
    MessageOperation,
    Status,
    SubscriptionId,
}
