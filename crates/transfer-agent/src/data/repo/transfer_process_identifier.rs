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

use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[allow(dead_code)]
#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferIdentifierRepoTrait: Send + Sync {
    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferProcessIdentifier>>;

    async fn get_identifiers_by_batch_process_id(
        &self,
        process_id_batch: &[Urn],
    ) -> Outcome<Vec<TransferProcessIdentifier>>;

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> Outcome<Option<TransferProcessIdentifier>>;

    async fn upsert_identifier(
        &self,
        process_id: &Urn,
        identifier: &TransferProcessIdentifier,
    ) -> Outcome<TransferProcessIdentifier>;

    async fn delete_identifier(&self, process_id: &Urn, key: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferIdentifierRepoErrors {
    #[error("Transfer Identifier not found")]
    TransferIdentifierNotFound,
    #[error("Error fetching transfer identifier. {0}")]
    ErrorFetchingTransferIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error upserting transfer identifier. {0}")]
    ErrorUpsertingTransferIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer identifier. {0}")]
    ErrorDeletingTransferIdentifier(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferIdentifierRepoErrors {}
