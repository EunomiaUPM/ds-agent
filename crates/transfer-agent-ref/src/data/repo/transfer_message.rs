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

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::entities::transfer_message::TransferMessage;
use common::query::{Page, Sort};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait TransferMessageRepoTrait: Send + Sync {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>>;
    async fn count_transfer_messages(&self, filters: &TransferMessageFilter) -> Outcome<u64>;

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>>;

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<Option<TransferMessage>>;

    async fn create_transfer_message(
        &self,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessage>;

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferMessageRepoErrors {
    #[allow(dead_code)]
    #[error("Transfer Message not found")]
    TransferMessageNotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching transfer message. {0}")]
    ErrorFetchingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer message. {0}")]
    ErrorCreatingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer message. {0}")]
    ErrorDeletingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferMessageRepoErrors {}
