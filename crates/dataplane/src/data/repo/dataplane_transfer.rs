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

use crate::data::sea_orm::orm::dataplane_transfers::{
    self, EditDataplaneTransferModel, NewDataplaneTransferModel,
};
use crate::entities::filters::DataplaneTransferFilter;
use common::query::{Page, Sort};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataplaneTransfersRepo: Send + Sync + 'static {
    async fn get_all_dataplane_transfers(
        &self,
        filters: &DataplaneTransferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<dataplane_transfers::Model>>;

    async fn count_dataplane_transfers(&self, filters: &DataplaneTransferFilter) -> Outcome<u64>;

    async fn get_batch_dataplane_transfers(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<dataplane_transfers::Model>>;

    async fn get_dataplane_transfers_by_id(
        &self,
        tenant_id: Option<String>,
        process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>>;

    /// Looks a transfer up by id in any tenant. Only for callers that hold the id as a
    /// capability (the data proxy); everything else must go through a tenant scope.
    async fn find_dataplane_transfer_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>>;

    async fn get_by_transfer_process_id(
        &self,
        tenant_id: Option<String>,
        transfer_process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>>;

    async fn create_dataplane_transfers(
        &self,
        new_dataplane_transfer: &NewDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model>;

    async fn put_dataplane_transfers(
        &self,
        tenant_id: Option<String>,
        process_id: &Urn,
        new_dataplane_transfer: &EditDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model>;

    async fn delete_dataplane_transfers(
        &self,
        tenant_id: Option<String>,
        process_id: &Urn,
    ) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum DataplaneTransfersRepoErrors {
    #[error("Dataplane transfer not found")]
    DataplaneTransferNotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching dataplane transfer. {0}")]
    ErrorFetchingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating dataplane transfer. {0}")]
    ErrorCreatingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting dataplane transfer. {0}")]
    ErrorDeletingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating dataplane transfer. {0}")]
    ErrorUpdatingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for DataplaneTransfersRepoErrors {}
