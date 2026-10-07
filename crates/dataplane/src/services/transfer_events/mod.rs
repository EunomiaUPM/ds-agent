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

//! Diagnostic events.

pub mod service;

pub use service::TransferEventsService;

use common::oauth::UserInfo;
use common::batch_requests::BatchRequests;
use common::query::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

use crate::entities::filters::TransferEventFilter;
use crate::entities::transfer_events::{NewTransferEventDto, TransferEventDto};

/// Recording and reading diagnostic events.
#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferEventServiceTrait: Send + Sync + 'static {
    /// Page of events visible to the caller.
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &TransferEventFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferEventDto>>;

    /// 404 when the event is not visible to the caller.
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<TransferEventDto>;

    /// Every event of the process.
    async fn get_by_process_id(
        &self,
        user: &UserInfo,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferEventDto>>;

    /// Events found among the requested ids.
    async fn batch(
        &self,
        user: &UserInfo,
        req: &BatchRequests,
    ) -> Outcome<Vec<TransferEventDto>>;

    async fn create(
        &self,
        user: &UserInfo,
        cmd: &NewTransferEventDto,
    ) -> Outcome<TransferEventDto>;
}
