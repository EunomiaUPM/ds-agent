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

use crate::data::entities::transfer_process;
use crate::data::entities::transfer_process::{EditTransferProcessModel, NewTransferProcessModel};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferProcessRepoTrait: Send + Sync {
    async fn get_all_transfer_processes(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_process::Model>>;
    async fn get_batch_transfer_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<transfer_process::Model>>;
    async fn get_transfer_process_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<transfer_process::Model>>;
    async fn get_transfer_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<transfer_process::Model>>;
    async fn get_transfer_process_by_key_value(
        &self,
        id: &Urn,
    ) -> Outcome<Option<transfer_process::Model>>;
    async fn create_transfer_process(
        &self,
        new_model: &NewTransferProcessModel,
    ) -> Outcome<transfer_process::Model>;
    async fn put_transfer_process(
        &self,
        id: &Urn,
        edit_model: &EditTransferProcessModel,
    ) -> Outcome<transfer_process::Model>;
    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferProcessRepoErrors {
    #[error("Transfer Process not found")]
    TransferProcessNotFound,
    #[error("Error fetching transfer process. {0}")]
    ErrorFetchingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer process. {0}")]
    ErrorCreatingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer process. {0}")]
    ErrorDeletingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating transfer process. {0}")]
    ErrorUpdatingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferProcessRepoErrors {}
