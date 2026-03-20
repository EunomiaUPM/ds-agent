/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use super::{DataplaneTransferLogDto, DataplaneTransferLogsEntitiesTrait};
use crate::data::factory_trait::DataplaneRepoTrait;
use common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::Outcome;

pub struct DataplaneTransferLogsEntityService {
    pub data_plane_repo: Arc<dyn DataplaneRepoTrait>,
}

impl DataplaneTransferLogsEntityService {
    pub fn new(data_plane_repo: Arc<dyn DataplaneRepoTrait>) -> Self {
        Self { data_plane_repo }
    }
}

#[async_trait::async_trait]
impl DataplaneTransferLogsEntitiesTrait for DataplaneTransferLogsEntityService {
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<DataplaneTransferLogDto>> {
        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_dataplane_process_id(&dataplane_process_id)
            .await?;

        Ok(logs.into_iter().map(|log| DataplaneTransferLogDto { inner: log }).collect())
    }
}
