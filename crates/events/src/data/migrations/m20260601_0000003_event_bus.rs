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

use crate::data::migrations::m20241123_0000001_subscriptions::Subscriptions;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260601_0000003_event_bus"
    }
}

#[derive(DeriveIden)]
pub enum Events {
    Table,
    Id,
    TenantId,
    Topic,
    SourceCrate,
    SchemaVersion,
    CorrelationId,
    Payload,
    Timestamp,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum EventDeliveries {
    Table,
    Id,
    TenantId,
    EventId,
    SubscriptionId,
    Status,
    Attempts,
    LastAttemptAt,
    NextRetryAt,
    ErrorMessage,
    ResponseStatusCode,
    DeliveredAt,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum DeadLetterQueue {
    Table,
    Id,
    TenantId,
    DeliveryId,
    EventId,
    SubscriptionId,
    Topic,
    CallbackAddress,
    Payload,
    ErrorMessage,
    Attempts,
    Status,
    FailedAt,
    ReplayedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create events table
        manager
            .create_table(
                Table::create()
                    .table(Events::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Events::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Events::TenantId).string().not_null())
                    .col(ColumnDef::new(Events::Topic).string().not_null())
                    .col(ColumnDef::new(Events::SourceCrate).string().not_null())
                    .col(
                        ColumnDef::new(Events::SchemaVersion)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Events::CorrelationId).string())
                    .col(ColumnDef::new(Events::Payload).json().not_null())
                    .col(ColumnDef::new(Events::Timestamp).date_time().not_null())
                    .col(ColumnDef::new(Events::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_events_topic")
                    .table(Events::Table)
                    .col(Events::Topic)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_events_timestamp")
                    .table(Events::Table)
                    .col(Events::Timestamp)
                    .to_owned(),
            )
            .await?;

        // 2. Create event_deliveries table
        manager
            .create_table(
                Table::create()
                    .table(EventDeliveries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EventDeliveries::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EventDeliveries::TenantId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EventDeliveries::EventId).string().not_null())
                    .col(
                        ColumnDef::new(EventDeliveries::SubscriptionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EventDeliveries::Status).string().not_null())
                    .col(
                        ColumnDef::new(EventDeliveries::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(EventDeliveries::LastAttemptAt).date_time())
                    .col(ColumnDef::new(EventDeliveries::NextRetryAt).date_time())
                    .col(ColumnDef::new(EventDeliveries::ErrorMessage).string())
                    .col(ColumnDef::new(EventDeliveries::ResponseStatusCode).integer())
                    .col(ColumnDef::new(EventDeliveries::DeliveredAt).date_time())
                    .col(
                        ColumnDef::new(EventDeliveries::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_deliveries_event")
                            .from(EventDeliveries::Table, EventDeliveries::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_deliveries_subscription")
                            .from(EventDeliveries::Table, EventDeliveries::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_deliveries_status_retry")
                    .table(EventDeliveries::Table)
                    .col(EventDeliveries::Status)
                    .col(EventDeliveries::NextRetryAt)
                    .to_owned(),
            )
            .await?;

        // 3. Create dead_letter_queue table
        manager
            .create_table(
                Table::create()
                    .table(DeadLetterQueue::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeadLetterQueue::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DeadLetterQueue::TenantId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeadLetterQueue::DeliveryId).string())
                    .col(ColumnDef::new(DeadLetterQueue::EventId).string().not_null())
                    .col(
                        ColumnDef::new(DeadLetterQueue::SubscriptionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeadLetterQueue::Topic).string().not_null())
                    .col(
                        ColumnDef::new(DeadLetterQueue::CallbackAddress)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeadLetterQueue::Payload).json().not_null())
                    .col(
                        ColumnDef::new(DeadLetterQueue::ErrorMessage)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeadLetterQueue::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DeadLetterQueue::Status)
                            .string()
                            .not_null()
                            .default("Unresolved"),
                    )
                    .col(
                        ColumnDef::new(DeadLetterQueue::FailedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeadLetterQueue::ReplayedAt).date_time())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_dlq_status_failed")
                    .table(DeadLetterQueue::Table)
                    .col(DeadLetterQueue::Status)
                    .col(DeadLetterQueue::FailedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DeadLetterQueue::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EventDeliveries::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Events::Table).to_owned())
            .await?;
        Ok(())
    }
}
