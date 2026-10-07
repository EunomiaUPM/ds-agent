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

//! Webhook subscription repository.

use async_trait::async_trait;
use common::oauth::{Owner, OwnerScope};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::queries::SubscriptionFilter;
use crate::entities::subscription::SubscriptionRecord;
use crate::entities::topic::Topic;
use common::paginated_spec::{Page, Sort};

/// Failures of the subscription repository, mapped onto `Errors`.
#[derive(Debug, Error)]
pub enum SubscriptionRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Subscription not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for SubscriptionRepoError {}

/// Persistence of webhook subscriptions and topic matching.
#[mockall::automock]
#[async_trait]
pub trait EventSubscriptionRepo: Send + Sync + 'static {
    /// Stores a subscription of `owner`.
    async fn create_subscription(
        &self,
        owner: Owner,
        dto: CreateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord>;
    async fn get_subscription(
        &self,
        scope: &OwnerScope,
        id: &str,
    ) -> Outcome<Option<SubscriptionRecord>>;
    /// Page of the subscriptions in `scope`.
    async fn list_subscriptions(
        &self,
        scope: &OwnerScope,
        filter: &SubscriptionFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<SubscriptionRecord>, u64)>;
    /// Changes the fields set in `dto`.
    async fn update_subscription(
        &self,
        scope: &OwnerScope,
        id: &str,
        dto: UpdateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord>;
    async fn delete_subscription(&self, scope: &OwnerScope, id: &str) -> Outcome<()>;
    /// Active subscriptions whose pattern matches `topic` and whose owner sees a record of
    /// `owner`, the one the event is about.
    async fn get_matching_subscriptions(
        &self,
        owner: &Owner,
        topic: &Topic,
    ) -> Outcome<Vec<SubscriptionRecord>>;
}
