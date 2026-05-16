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

pub(crate) mod service;
#[cfg(test)]
mod tests;
pub(crate) mod views;

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Paginated, Sort, TransferMessageFilter};
use crate::services::transfer_message::views::TransferMessageView;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub(crate) trait TransferMessageServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>>;

    async fn get_all_by_process(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>>;

    async fn get_one(&self, id: &Urn) -> Outcome<TransferMessageView>;

    async fn create(&self, cmd: &NewTransferMessageCommand) -> Outcome<TransferMessageView>;

    async fn delete(&self, id: &Urn) -> Outcome<()>;
}
