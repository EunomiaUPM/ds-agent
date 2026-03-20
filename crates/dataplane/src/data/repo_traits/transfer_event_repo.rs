/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::transfer_event;
use crate::data::entities::transfer_event::NewTransferEvent;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[async_trait::async_trait]
pub trait TransferEventRepo: Send + Sync + 'static {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_event::Model>>;
    async fn get_batch_transfer_events(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<transfer_event::Model>>;
    async fn get_all_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_event::Model>>;
    async fn get_transfer_event_by_id(
        &self,
        transfer_event: &Urn,
    ) -> Outcome<Option<transfer_event::Model>>;
    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEvent,
    ) -> Outcome<transfer_event::Model>;
}

#[derive(Debug, Error)]
pub enum TransferEventRepoErrors {
    #[error("Transfer event not found")]
    TransferEventNotFound,
    #[error("Error fetching transfer event. {0}")]
    ErrorFetchingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer event. {0}")]
    ErrorCreatingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer event. {0}")]
    ErrorDeletingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating transfer event. {0}")]
    ErrorUpdatingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferEventRepoErrors {}
