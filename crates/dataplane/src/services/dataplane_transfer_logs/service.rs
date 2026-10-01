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

use std::sync::Arc;

use common::auth::access::AccessScope;
use urn::Urn;
use ymir::errors::Outcome;

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::dataplane_transfer_logs::DataplaneTransferLogDto;
use crate::services::dataplane_transfer_logs::DataplaneTransferLogServiceTrait;

pub struct DataplaneTransferLogsService {
    data_plane_repo: Arc<dyn DataplaneRepoTrait>,
}

impl DataplaneTransferLogsService {
    pub fn new(data_plane_repo: Arc<dyn DataplaneRepoTrait>) -> Self {
        Self { data_plane_repo }
    }
}

#[async_trait::async_trait]
impl DataplaneTransferLogServiceTrait for DataplaneTransferLogsService {
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        scope: &AccessScope,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<DataplaneTransferLogDto>> {
        scope.require_read()?;

        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_dataplane_process_id(
                scope.tenant_filter().map(str::to_string),
                dataplane_process_id,
            )
            .await?;

        Ok(logs
            .into_iter()
            .map(|log| DataplaneTransferLogDto { inner: log })
            .collect())
    }
}
