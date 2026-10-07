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

//! DCAT data service management use cases.

pub mod service;

use crate::entities::data_services::{DataServiceDto, EditDataServiceDto, NewDataServiceDto};
use crate::entities::filters::DataServiceFilter;
use common::oauth::UserInfo;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Management of data services.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DataServiceServiceTrait: Send + Sync {
    /// Page of data services visible to the caller.
    async fn get_all_data_services(
        &self,
        user: &UserInfo,
        filters: &DataServiceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DataServiceDto>>;
    /// Data services found among `ids`.
    async fn get_batch_data_services(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<DataServiceDto>>;

    /// Data services of the catalog.
    async fn get_data_services_by_catalog_id(
        &self,
        user: &UserInfo,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DataServiceDto>>;

    /// The connector's main data service, if it has one (whoever asks).
    async fn get_main_data_service(&self, user: &UserInfo) -> Outcome<Option<DataServiceDto>>;
    /// 404 when the data service is not visible to the caller.
    async fn get_data_service_by_id(
        &self,
        user: &UserInfo,
        data_service_id: &Urn,
    ) -> Outcome<DataServiceDto>;
    async fn put_data_service_by_id(
        &self,
        user: &UserInfo,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn create_data_service(
        &self,
        user: &UserInfo,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    /// Creates the connector's main data service (the root only), unless there is one already.
    async fn create_main_data_service(
        &self,
        user: &UserInfo,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn delete_data_service_by_id(
        &self,
        user: &UserInfo,
        data_service_id: &Urn,
    ) -> Outcome<()>;
}
