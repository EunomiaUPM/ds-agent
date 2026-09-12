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

use async_trait::async_trait;
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic::Topic;

// Repository errors encountered during webhook subscription management.
#[derive(Debug, Error)]
pub enum SubscriptionRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Subscription not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for SubscriptionRepoError {}

// Repository interface for webhook subscription CRUD and matching.
#[async_trait]
pub trait EventSubscriptionRepo: Send + Sync + 'static {
    async fn create_subscription(&self, dto: CreateSubscriptionDto) -> Outcome<SubscriptionRecord>;
    async fn get_subscription(&self, id: &str) -> Outcome<Option<SubscriptionRecord>>;
    async fn list_subscriptions(&self) -> Outcome<Vec<SubscriptionRecord>>;
    async fn update_subscription(
        &self,
        id: &str,
        dto: UpdateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord>;
    async fn delete_subscription(&self, id: &str) -> Outcome<()>;
    async fn get_matching_subscriptions(&self, topic: &Topic) -> Outcome<Vec<SubscriptionRecord>>;
}
