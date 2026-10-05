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

//! Delivery attempt repository.

use async_trait::async_trait;
use common::oauth::OwnerScope;
use chrono::{DateTime, Utc};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::delivery::EventDeliveryRecord;

/// Failures of the delivery repository, mapped onto `Errors`.
#[derive(Debug, Error)]
pub enum DeliveryRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Delivery record not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for DeliveryRepoError {}

/// Persistence of webhook delivery attempts and their retry schedule.
#[mockall::automock]
#[async_trait]
pub trait EventDeliveryRepo: Send + Sync + 'static {
    async fn create_delivery(&self, delivery: &EventDeliveryRecord)
        -> Outcome<EventDeliveryRecord>;
    async fn get_delivery(&self, scope: &OwnerScope, id: &str)
        -> Outcome<Option<EventDeliveryRecord>>;
    /// Pending deliveries whose next retry is due by `now`, at most `limit`.
    async fn get_due_retries(
        &self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Outcome<Vec<EventDeliveryRecord>>;
    async fn mark_delivered(&self, id: &str, attempts: u32, status_code: u16) -> Outcome<()>;
    /// Records a failed attempt; `next_retry_at` of `None` stops retrying.
    async fn record_failed_attempt(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: Option<DateTime<Utc>>,
        error: &str,
        status_code: Option<u16>,
    ) -> Outcome<()>;
    async fn mark_dead_letter(&self, id: &str) -> Outcome<()>;
    /// Every delivery of the event in `scope`.
    async fn list_by_event(
        &self,
        scope: &OwnerScope,
        event_id: &str,
    ) -> Outcome<Vec<EventDeliveryRecord>>;
}
