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

use crate::data::entities::dataplane_field;
use crate::data::entities::dataplane_field::{EditDataPlaneFieldModel, NewDataPlaneFieldModel};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[async_trait::async_trait]
pub trait DataplaneFieldRepoTrait: Send + Sync + 'static {
    async fn get_all_dataplane_fields(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataplane_field::Model>>;
    async fn get_batch_dataplane_fields(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<dataplane_field::Model>>;
    async fn get_all_dataplane_fields_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<dataplane_field::Model>>;
    async fn get_dataplane_field_by_id(
        &self,
        field_id: &Urn,
    ) -> Outcome<Option<dataplane_field::Model>>;
    async fn create_dataplane_field(
        &self,
        process_id: &Urn,
        new_dataplane_field: &NewDataPlaneFieldModel,
    ) -> Outcome<dataplane_field::Model>;
    async fn put_dataplane_field(
        &self,
        field_id: &Urn,
        edit_field: &EditDataPlaneFieldModel,
    ) -> Outcome<dataplane_field::Model>;
    async fn delete_dataplane_field(&self, field_id: &Urn) -> Outcome<()>;
    async fn delete_all_dataplane_fields_by_process_id(&self, process_id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum DataplaneFieldRepoErrors {
    #[error("Dataplane field not found")]
    DataplaneFieldNotFound,
    #[error("Error fetching dataplane field. {0}")]
    ErrorFetchingDataplaneField(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating dataplane field. {0}")]
    ErrorCreatingDataplaneField(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting dataplane field. {0}")]
    ErrorDeletingDataplaneField(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating dataplane field. {0}")]
    ErrorUpdatingDataplaneField(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for DataplaneFieldRepoErrors {}
