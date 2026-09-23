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

use crate::data::repo::dataplane_field::DataplaneFieldRepoTrait;
use crate::data::repo::dataplane_transfer::DataplaneTransfersRepo;
use crate::data::repo::dataplane_transfer_log::DataplaneTransferLogsRepo;
use crate::data::repo::transfer_event::TransferEventRepo;
use std::sync::Arc;

#[mockall::automock]
pub trait DataplaneRepoTrait: Send + Sync + 'static {
    fn get_dataplane_transfers_repo(&self) -> Arc<dyn DataplaneTransfersRepo>;
    fn get_dataplane_fields_repo(&self) -> Arc<dyn DataplaneFieldRepoTrait>;
    fn get_dataplane_transfer_logs_repo(&self) -> Arc<dyn DataplaneTransferLogsRepo>;
    fn get_transfer_events_repo(&self) -> Arc<dyn TransferEventRepo>;
}
