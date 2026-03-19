/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::transfer_process_identifier;
use crate::data::entities::transfer_process_identifier::{
    EditTransferIdentifierModel, NewTransferIdentifierModel,
};
use anyhow::Error;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
#[allow(unused)]
pub trait TransferIdentifierRepoTrait: Send + Sync {
    async fn get_all_identifiers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_process_identifier::Model>>;

    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_process_identifier::Model>>;

    async fn get_identifier_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<transfer_process_identifier::Model>>;

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> Outcome<Option<transfer_process_identifier::Model>>;

    async fn create_identifier(
        &self,
        new_model: &NewTransferIdentifierModel,
    ) -> Outcome<transfer_process_identifier::Model>;

    async fn put_identifier(
        &self,
        id: &Urn,
        edit_model: &EditTransferIdentifierModel,
    ) -> Outcome<transfer_process_identifier::Model>;

    async fn delete_identifier(&self, id: &Urn)
        -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferIdentifierRepoErrors {
    #[error("Transfer Identifier not found")]
    TransferIdentifierNotFound,
    #[error("Error fetching transfer identifier. {0}")]
    ErrorFetchingTransferIdentifier(Error),
    #[error("Error creating transfer identifier. {0}")]
    ErrorCreatingTransferIdentifier(Error),
    #[error("Error deleting transfer identifier. {0}")]
    ErrorDeletingTransferIdentifier(Error),
    #[error("Error updating transfer identifier. {0}")]
    ErrorUpdatingTransferIdentifier(Error),
}

impl RepoIntoErrors for TransferIdentifierRepoErrors {}
