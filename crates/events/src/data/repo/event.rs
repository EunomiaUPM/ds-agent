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
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::envelope::EventEnvelope;
use crate::entities::queries::EventFilter;
use common::paginated_spec::{Page, Sort};

// Repository errors encountered during event store persistence.
#[derive(Debug, Error)]
pub enum EventRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Event not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for EventRepoError {}

// Repository interface for immutable event storage.
#[async_trait]
pub trait EventStoreRepo: Send + Sync + 'static {
    async fn insert_event(&self, event: &EventEnvelope) -> Outcome<()>;
    /// `tenant_id: None` reads across tenants (admin).
    async fn get_event_by_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<EventEnvelope>>;
    /// A page of events plus the total count of the filtered set.
    async fn list_events(
        &self,
        tenant_id: Option<String>,
        filter: &EventFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<EventEnvelope>, u64)>;
}
