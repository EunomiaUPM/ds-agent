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
use anyhow::Error;
use thiserror::Error;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
#[allow(unused)]
pub trait NegotiationIdentifierRepoTrait: Send + Sync {
    async fn get_all_identifiers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<negotiation_process_identifier::Model>, NegotiationIdentifierRepoErrors>;

    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<negotiation_process_identifier::Model>, NegotiationIdentifierRepoErrors>;

    async fn get_identifier_by_id(
        &self,
        id: &Urn,
    ) -> anyhow::Result<
        Option<negotiation_process_identifier::Model>,
        NegotiationIdentifierRepoErrors,
    >;

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> anyhow::Result<
        Option<negotiation_process_identifier::Model>,
        NegotiationIdentifierRepoErrors,
    >;

    async fn create_identifier(
        &self,
        new_model: &NewNegotiationIdentifierModel,
    ) -> anyhow::Result<negotiation_process_identifier::Model, NegotiationIdentifierRepoErrors>;

    async fn put_identifier(
        &self,
        id: &Urn,
        edit_model: &EditNegotiationIdentifierModel,
    ) -> anyhow::Result<negotiation_process_identifier::Model, NegotiationIdentifierRepoErrors>;

    async fn delete_identifier(
        &self,
        id: &Urn,
    ) -> anyhow::Result<(), NegotiationIdentifierRepoErrors>;
}

#[derive(Debug, Error)]
pub enum NegotiationIdentifierRepoErrors {
    #[error("Negotiation Identifier not found")]
    NegotiationIdentifierNotFound,
    #[error("Error fetching negotiation identifier. {0}")]
    ErrorFetchingNegotiationIdentifier(Error),
    #[error("Error creating negotiation identifier. {0}")]
    ErrorCreatingNegotiationIdentifier(Error),
    #[error("Error deleting negotiation identifier. {0}")]
    ErrorDeletingNegotiationIdentifier(Error),
    #[error("Error updating negotiation identifier. {0}")]
    ErrorUpdatingNegotiationIdentifier(Error),
}

impl RepoIntoErrors for NegotiationIdentifierRepoErrors {}
