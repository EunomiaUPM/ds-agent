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

//! Data service repository.

use crate::data::entities::dataservice;
use common::oauth::OwnerScope;
use crate::data::entities::dataservice::{EditDataServiceModel, NewDataServiceModel};
use crate::entities::filters::DataServiceFilter;
use common::paginated_spec::{Page, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Persistence of data services, within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DataServiceRepositoryTrait: Send + Sync {
    /// Page of data services matching the filters, with the total.
    async fn get_all_data_services(
        &self,
        scope: &OwnerScope,
        filters: &DataServiceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<dataservice::Model>, Option<u64>)>;
    async fn get_batch_data_services(
        &self,
        scope: &OwnerScope,
        ids: &[Urn],
    ) -> Outcome<Vec<dataservice::Model>>;

    /// Data services of the catalog.
    async fn get_data_services_by_catalog_id(
        &self,
        scope: &OwnerScope,
        catalog_id: &Urn,
    ) -> Outcome<Vec<dataservice::Model>>;
    /// The connector's main data service.
    async fn get_main_data_service(&self) -> Outcome<Option<dataservice::Model>>;

    async fn get_data_service_by_id(
        &self,
        scope: &OwnerScope,
        data_service_id: &Urn,
    ) -> Outcome<Option<dataservice::Model>>;
    async fn put_data_service_by_id(
        &self,
        scope: &OwnerScope,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceModel,
    ) -> Outcome<dataservice::Model>;
    async fn create_data_service(
        &self,
        new_data_service_model: &NewDataServiceModel,
    ) -> Outcome<dataservice::Model>;

    /// Stores the data service as the connector's main one, unless there is one already.
    async fn create_main_data_service(
        &self,
        new_data_service_model: &NewDataServiceModel,
    ) -> Outcome<dataservice::Model>;
    /// Deletes and returns the removed row so callers can evict derived caches.
    async fn delete_data_service_by_id(
        &self,
        scope: &OwnerScope,
        data_service_id: &Urn,
    ) -> Outcome<dataservice::Model>;
}
