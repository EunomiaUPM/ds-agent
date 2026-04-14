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

use crate::data::entities::dataplane_transfer_logs;
use async_trait::async_trait;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[async_trait]
pub trait DataplaneTransferLogsRepo: Send + Sync {
    async fn create_log(
        &self,
        new_log: dataplane_transfer_logs::NewTransferLog,
    ) -> Outcome<dataplane_transfer_logs::Model>;
    async fn get_all_transfer_logs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataplane_transfer_logs::Model>>;
    async fn get_transfer_log_by_id(
        &self,
        log_id: &Urn,
    ) -> Outcome<Option<dataplane_transfer_logs::Model>>;
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<dataplane_transfer_logs::Model>>;
}

#[derive(Debug, Error)]
pub enum DataplaneTransferLogsRepoErrors {
    #[error("Dataplane transfer log not found")]
    DataplaneTransferLogNotFound,
    #[error("Error fetching dataplane transfer log. {0}")]
    ErrorFetchingDataplaneTransferLog(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating dataplane transfer log. {0}")]
    ErrorCreatingDataplaneTransferLog(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting dataplane transfer log. {0}")]
    ErrorDeletingDataplaneTransferLog(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating dataplane transfer log. {0}")]
    ErrorUpdatingDataplaneTransferLog(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for DataplaneTransferLogsRepoErrors {}
