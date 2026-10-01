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
use common::auth::AccessScope;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataServiceServiceTrait: Send + Sync {
    async fn get_all_data_services(
        &self,
        scope: &AccessScope,
        filters: &DataServiceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DataServiceDto>>;
    async fn get_batch_data_services(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<DataServiceDto>>;

    async fn get_data_services_by_catalog_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DataServiceDto>>;

    async fn get_main_data_service(&self, scope: &AccessScope) -> Outcome<Option<DataServiceDto>>;
    async fn get_data_service_by_id(
        &self,
        scope: &AccessScope,
        data_service_id: &Urn,
    ) -> Outcome<DataServiceDto>;
    async fn put_data_service_by_id(
        &self,
        scope: &AccessScope,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn create_data_service(
        &self,
        scope: &AccessScope,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn create_main_data_service(
        &self,
        scope: &AccessScope,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto>;
    async fn delete_data_service_by_id(
        &self,
        scope: &AccessScope,
        data_service_id: &Urn,
    ) -> Outcome<()>;
}
