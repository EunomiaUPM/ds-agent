/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::transfer_message;
use crate::data::entities::transfer_message::NewTransferMessageModel;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferMessageRepoTrait: Send + Sync {
    // Obtener todos (paginado)
    async fn get_all_transfer_messages(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_message::Model>>;

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_message::Model>>;

    async fn get_transfer_message_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<transfer_message::Model>>;

    async fn create_transfer_message(
        &self,
        new_model: &NewTransferMessageModel,
    ) -> Outcome<transfer_message::Model>;

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferMessageRepoErrors {
    #[error("Transfer Message not found")]
    TransferMessageNotFound,
    #[error("Error fetching transfer message. {0}")]
    ErrorFetchingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer message. {0}")]
    ErrorCreatingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer message. {0}")]
    ErrorDeletingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferMessageRepoErrors {}
