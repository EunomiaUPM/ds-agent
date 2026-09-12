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
use chrono::{DateTime, Utc};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::delivery::EventDeliveryRecord;

// Repository errors encountered during delivery tracking.
#[derive(Debug, Error)]
pub enum DeliveryRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Delivery record not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for DeliveryRepoError {}

// Repository interface for webhook delivery attempt tracking and retries.
#[async_trait]
pub trait EventDeliveryRepo: Send + Sync + 'static {
    async fn create_delivery(&self, delivery: &EventDeliveryRecord)
        -> Outcome<EventDeliveryRecord>;
    async fn get_delivery(&self, id: &str) -> Outcome<Option<EventDeliveryRecord>>;
    async fn get_due_retries(
        &self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Outcome<Vec<EventDeliveryRecord>>;
    async fn mark_delivered(&self, id: &str, attempts: u32, status_code: u16) -> Outcome<()>;
    async fn record_failed_attempt(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: Option<DateTime<Utc>>,
        error: &str,
        status_code: Option<u16>,
    ) -> Outcome<()>;
    async fn mark_dead_letter(&self, id: &str) -> Outcome<()>;
    async fn list_by_event(&self, event_id: &str) -> Outcome<Vec<EventDeliveryRecord>>;
}
