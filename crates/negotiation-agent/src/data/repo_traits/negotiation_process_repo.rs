/*
 *
 * * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::negotiation_process;
use crate::data::entities::negotiation_process::{
    EditNegotiationProcessModel, NewNegotiationProcessModel,
};
use thiserror::Error;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
pub trait NegotiationProcessRepoTrait: Send + Sync {
    async fn get_all_negotiation_processes(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<negotiation_process::Model>>;
    async fn get_batch_negotiation_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<negotiation_process::Model>>;
    async fn get_negotiation_process_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<negotiation_process::Model>>;
    async fn get_negotiation_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<negotiation_process::Model>>;
    async fn get_negotiation_process_by_key_value(
        &self,
        id: &Urn,
    ) -> Outcome<Option<negotiation_process::Model>>;
    async fn create_negotiation_process(
        &self,
        new_model: &NewNegotiationProcessModel,
    ) -> Outcome<negotiation_process::Model>;
    async fn put_negotiation_process(
        &self,
        id: &Urn,
        edit_model: &EditNegotiationProcessModel,
    ) -> Outcome<negotiation_process::Model>;
    async fn delete_negotiation_process(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum NegotiationProcessRepoErrors {
    #[error("Negotiation Process not found")]
    NegotiationProcessNotFound,
    #[error("Error fetching negotiation process. {0}")]
    ErrorFetchingNegotiationProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating negotiation process. {0}")]
    ErrorCreatingNegotiationProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting negotiation process. {0}")]
    ErrorDeletingNegotiationProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating negotiation process. {0}")]
    ErrorUpdatingNegotiationProcess(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for NegotiationProcessRepoErrors {}
