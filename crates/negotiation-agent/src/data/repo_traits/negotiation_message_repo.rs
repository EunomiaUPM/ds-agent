/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::negotiation_message;
use crate::data::entities::negotiation_message::NewNegotiationMessageModel;
use thiserror::Error;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
pub trait NegotiationMessageRepoTrait: Send + Sync {
    async fn get_all_negotiation_messages(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<negotiation_message::Model>>;

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<negotiation_message::Model>>;

    async fn get_negotiation_message_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<negotiation_message::Model>>;

    async fn create_negotiation_message(
        &self,
        new_model: &NewNegotiationMessageModel,
    ) -> Outcome<negotiation_message::Model>;

    async fn delete_negotiation_message(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum NegotiationMessageRepoErrors {
    #[error("Negotiation Message not found")]
    NegotiationMessageNotFound,
    #[error("Error fetching negotiation message. {0}")]
    ErrorFetchingNegotiationMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating negotiation message. {0}")]
    ErrorCreatingNegotiationMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting negotiation message. {0}")]
    ErrorDeletingNegotiationMessage(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for NegotiationMessageRepoErrors {}
