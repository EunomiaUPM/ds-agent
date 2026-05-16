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

use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::services::transfer_process::views::TransferProcessView;
use common::batch_requests::BatchRequests;
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod service;
pub(crate) mod views;

#[async_trait::async_trait]
pub(crate) trait TransferProcessServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferProcessView>>;
    async fn get_one(&self, id: &Urn) -> Outcome<TransferProcessView>;
    async fn batch(&self, batch_request: &BatchRequests) -> Outcome<Vec<TransferProcessView>>;
    async fn create(
        &self,
        cmd: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcessView>;
    async fn edit(
        &self,
        id: &Urn,
        cmd: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcessView>;
    async fn delete(&self, id: &Urn) -> Outcome<()>;
}
