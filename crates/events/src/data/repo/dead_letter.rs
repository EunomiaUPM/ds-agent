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

use crate::entities::dead_letter::DeadLetterRecord;

// Repository errors encountered during Dead Letter Queue operations.
#[derive(Debug, Error)]
pub enum DlqRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Dead letter record not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for DlqRepoError {}

// Repository interface for Dead Letter Queue persistence and re-drive.
#[async_trait]
pub trait EventDeadLetterRepo: Send + Sync + 'static {
    async fn create_dead_letter(&self, record: &DeadLetterRecord) -> Outcome<DeadLetterRecord>;
    async fn get_dead_letter(&self, id: &str) -> Outcome<Option<DeadLetterRecord>>;
    async fn list_dead_letters(
        &self,
        status: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<DeadLetterRecord>>;
    async fn mark_replayed(&self, id: &str) -> Outcome<()>;
    async fn delete_dead_letter(&self, id: &str) -> Outcome<()>;
}
