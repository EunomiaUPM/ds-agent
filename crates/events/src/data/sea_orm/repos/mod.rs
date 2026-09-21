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

pub mod dead_letter;
pub mod delivery;
pub mod event;
pub mod subscription;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use urn::Urn;
use ymir::errors::Outcome;

pub use dead_letter::SeaOrmDeadLetterRepo;
pub use delivery::SeaOrmDeliveryRepo;
pub use event::SeaOrmEventRepo;
pub use subscription::SeaOrmSubscriptionRepo;

use crate::data::repo::{
    EventDeadLetterRepo, EventDeliveryRepo, EventStoreRepo, EventSubscriptionRepo,
};
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::EventEnvelope;
use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic::Topic;

// Consolidated SeaORM repository facade for backward compatibility.
#[derive(Clone)]
pub struct SeaOrmEventBusRepo {
    event_repo: SeaOrmEventRepo,
    subscription_repo: SeaOrmSubscriptionRepo,
    delivery_repo: SeaOrmDeliveryRepo,
    dlq_repo: SeaOrmDeadLetterRepo,
}

impl SeaOrmEventBusRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            event_repo: SeaOrmEventRepo::new(db.clone()),
            subscription_repo: SeaOrmSubscriptionRepo::new(db.clone()),
            delivery_repo: SeaOrmDeliveryRepo::new(db.clone()),
            dlq_repo: SeaOrmDeadLetterRepo::new(db),
        }
    }
}

#[async_trait]
impl EventStoreRepo for SeaOrmEventBusRepo {
    async fn insert_event(&self, event: &EventEnvelope) -> Outcome<()> {
        self.event_repo.insert_event(event).await
    }

    async fn get_event_by_id(&self, tenant_id: &str, id: &Urn) -> Outcome<Option<EventEnvelope>> {
        self.event_repo.get_event_by_id(tenant_id, id).await
    }

    async fn list_events(
        &self,
        tenant_id: &str,
        topic: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<EventEnvelope>> {
        self.event_repo
            .list_events(tenant_id, topic, limit, offset)
            .await
    }
}

#[async_trait]
impl EventSubscriptionRepo for SeaOrmEventBusRepo {
    async fn create_subscription(
        &self,
        tenant_id: &str,
        dto: CreateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord> {
        self.subscription_repo
            .create_subscription(tenant_id, dto)
            .await
    }

    async fn get_subscription(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> Outcome<Option<SubscriptionRecord>> {
        self.subscription_repo.get_subscription(tenant_id, id).await
    }

    async fn list_subscriptions(&self, tenant_id: &str) -> Outcome<Vec<SubscriptionRecord>> {
        self.subscription_repo.list_subscriptions(tenant_id).await
    }

    async fn update_subscription(
        &self,
        tenant_id: &str,
        id: &str,
        dto: UpdateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord> {
        self.subscription_repo
            .update_subscription(tenant_id, id, dto)
            .await
    }

    async fn delete_subscription(&self, tenant_id: &str, id: &str) -> Outcome<()> {
        self.subscription_repo
            .delete_subscription(tenant_id, id)
            .await
    }

    async fn get_matching_subscriptions(
        &self,
        tenant_id: &str,
        topic: &Topic,
    ) -> Outcome<Vec<SubscriptionRecord>> {
        self.subscription_repo
            .get_matching_subscriptions(tenant_id, topic)
            .await
    }
}

#[async_trait]
impl EventDeliveryRepo for SeaOrmEventBusRepo {
    async fn create_delivery(
        &self,
        delivery: &EventDeliveryRecord,
    ) -> Outcome<EventDeliveryRecord> {
        self.delivery_repo.create_delivery(delivery).await
    }

    async fn get_delivery(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> Outcome<Option<EventDeliveryRecord>> {
        self.delivery_repo.get_delivery(tenant_id, id).await
    }

    async fn get_due_retries(
        &self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Outcome<Vec<EventDeliveryRecord>> {
        self.delivery_repo.get_due_retries(now, limit).await
    }

    async fn mark_delivered(&self, id: &str, attempts: u32, status_code: u16) -> Outcome<()> {
        self.delivery_repo
            .mark_delivered(id, attempts, status_code)
            .await
    }

    async fn record_failed_attempt(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: Option<DateTime<Utc>>,
        error: &str,
        status_code: Option<u16>,
    ) -> Outcome<()> {
        self.delivery_repo
            .record_failed_attempt(id, attempts, next_retry_at, error, status_code)
            .await
    }

    async fn mark_dead_letter(&self, id: &str) -> Outcome<()> {
        self.delivery_repo.mark_dead_letter(id).await
    }

    async fn list_by_event(
        &self,
        tenant_id: &str,
        event_id: &str,
    ) -> Outcome<Vec<EventDeliveryRecord>> {
        self.delivery_repo.list_by_event(tenant_id, event_id).await
    }
}

#[async_trait]
impl EventDeadLetterRepo for SeaOrmEventBusRepo {
    async fn create_dead_letter(&self, record: &DeadLetterRecord) -> Outcome<DeadLetterRecord> {
        self.dlq_repo.create_dead_letter(record).await
    }

    async fn get_dead_letter(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> Outcome<Option<DeadLetterRecord>> {
        self.dlq_repo.get_dead_letter(tenant_id, id).await
    }

    async fn list_dead_letters(
        &self,
        tenant_id: &str,
        status: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<DeadLetterRecord>> {
        self.dlq_repo
            .list_dead_letters(tenant_id, status, limit, offset)
            .await
    }

    async fn mark_replayed(&self, tenant_id: &str, id: &str) -> Outcome<()> {
        self.dlq_repo.mark_replayed(tenant_id, id).await
    }

    async fn delete_dead_letter(&self, tenant_id: &str, id: &str) -> Outcome<()> {
        self.dlq_repo.delete_dead_letter(tenant_id, id).await
    }
}
