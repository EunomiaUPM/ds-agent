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

pub mod service;

pub use service::DataplaneTransferLogsService;

use common::auth::access::AccessScope;
use urn::Urn;
use ymir::errors::Outcome;

use crate::entities::dataplane_transfer_logs::DataplaneTransferLogDto;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataplaneTransferLogServiceTrait: Send + Sync + 'static {
    async fn get_transfer_logs_by_dataplane_process_id(
        &self,
        scope: &AccessScope,
        dataplane_process_id: &Urn,
    ) -> Outcome<Vec<DataplaneTransferLogDto>>;
}
