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

use crate::data::entities::negotiation_process_identifier;
use crate::data::entities::negotiation_process_identifier::{
    EditNegotiationIdentifierModel, NewNegotiationIdentifierModel,
};
use thiserror::Error;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
#[allow(unused)]
pub trait NegotiationIdentifierRepoTrait: Send + Sync {
    async fn get_all_identifiers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<negotiation_process_identifier::Model>>;

    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<negotiation_process_identifier::Model>>;

    async fn get_identifier_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<negotiation_process_identifier::Model>>;

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> Outcome<Option<negotiation_process_identifier::Model>>;

    async fn create_identifier(
        &self,
        new_model: &NewNegotiationIdentifierModel,
    ) -> Outcome<negotiation_process_identifier::Model>;

    async fn put_identifier(
        &self,
        id: &Urn,
        edit_model: &EditNegotiationIdentifierModel,
    ) -> Outcome<negotiation_process_identifier::Model>;

    async fn delete_identifier(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum NegotiationIdentifierRepoErrors {
    #[error("Negotiation Identifier not found")]
    NegotiationIdentifierNotFound,
    #[error("Error fetching negotiation identifier. {0}")]
    ErrorFetchingNegotiationIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating negotiation identifier. {0}")]
    ErrorCreatingNegotiationIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting negotiation identifier. {0}")]
    ErrorDeletingNegotiationIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating negotiation identifier. {0}")]
    ErrorUpdatingNegotiationIdentifier(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for NegotiationIdentifierRepoErrors {}
