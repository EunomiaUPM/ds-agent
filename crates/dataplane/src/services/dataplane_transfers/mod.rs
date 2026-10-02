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

//! Dataplane processes.

pub mod service;

pub use service::DataplaneTransferService;

use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::query::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, EditDataplaneTransferDto, NewDataplaneTransferDto,
};
use crate::entities::filters::DataplaneTransferFilter;

/// Management of dataplane processes.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DataplaneTransferServiceTrait: Send + Sync + 'static {
    /// Page of processes visible to the caller.
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &DataplaneTransferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DataplaneTransferDto>>;

    /// 404 when the process is not visible to the caller.
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<DataplaneTransferDto>;

    /// Process serving the given transfer process.
    async fn get_by_process_id(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
    ) -> Outcome<DataplaneTransferDto>;

    /// Processes found among the requested ids.
    async fn batch(
        &self,
        scope: &AccessScope,
        req: &BatchRequests,
    ) -> Outcome<Vec<DataplaneTransferDto>>;

    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto>;

    /// Changes the fields set in `cmd`.
    async fn edit(
        &self,
        scope: &AccessScope,
        id: &Urn,
        cmd: &EditDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto>;

    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()>;
}
