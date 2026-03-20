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

use crate::data::entities::dataplane_transfer_logs;
use serde::{Deserialize, Serialize};
use urn::Urn;
use uuid::Uuid;
use ymir::errors::Outcome;

pub mod dataplane_transfer_logs_entity;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataplaneTransferLogDto {
    #[serde(flatten)]
    pub inner: dataplane_transfer_logs::Model,
}

#[async_trait::async_trait]
pub trait DataplaneTransferLogsEntitiesTrait: Send + Sync + 'static {
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<DataplaneTransferLogDto>>;
}
