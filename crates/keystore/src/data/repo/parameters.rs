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
use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::key::Key;
use crate::entities::version::Version;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};
use crate::entities::entry::Entry;

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock(type Value = serde_json::Value;))]
#[async_trait::async_trait]
pub trait ParameterRepoTrait: Send + Sync {
    type Value: Serialize + DeserializeOwned + Send + Sync + 'static;

    async fn get_all_parameters(&self) -> Outcome<Vec<Entry<Self::Value>>>;
    async fn count_parameters(&self) -> Outcome<u64>;
    async fn get_batch_parameters(
        &self,
        keys: &[Key],
    ) -> Outcome<Vec<Entry<Self::Value>>>;
    async fn get_parameter_by_key(
        &self,
        key: &Key,
    ) -> Outcome<Option<Entry<Self::Value>>>;
    async fn create_parameter(
        &self,
        new_model: &NewParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>>;
    async fn put_parameter(
        &self,
        key: &Key,
        edit_model: &EditParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>>;
    async fn delete_parameter(&self, key: &Key) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum ParameterRepoErrors {
    #[error("Parameter not found")]
    ParameterNotFound,
    #[error("Parameter already exists")]
    ParameterAlreadyExists,
    #[error("Version conflict: expected {expected:?}, actual {actual:?}")]
    VersionConflict { expected: Version, actual: Version },
    #[error("Error fetching parameter. {0}")]
    ErrorFetchingParameter(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating parameter. {0}")]
    ErrorCreatingParameter(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting parameter. {0}")]
    ErrorDeletingParameter(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating parameter. {0}")]
    ErrorUpdatingParameter(Box<dyn std::error::Error + Send + Sync>),
    #[error("Parameter deserialization failed. {0}")]
    ErrorDeserializingParameter(#[source] serde_json::Error),
}

impl RepoIntoErrors for ParameterRepoErrors {}