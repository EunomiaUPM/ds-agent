/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::dataservice;
use crate::data::entities::dataservice::{EditDataServiceModel, NewDataServiceModel};
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait DataServiceRepositoryTrait: Send + Sync {
    async fn get_all_data_services(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataservice::Model>>;
    async fn get_batch_data_services(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<dataservice::Model>>;

    async fn get_data_services_by_catalog_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Vec<dataservice::Model>>;
    async fn get_main_data_service(
        &self,
    ) -> Outcome<Option<dataservice::Model>>;

    async fn get_data_service_by_id(
        &self,
        data_service_id: &Urn,
    ) -> Outcome<Option<dataservice::Model>>;
    async fn put_data_service_by_id(
        &self,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceModel,
    ) -> Outcome<dataservice::Model>;
    async fn create_data_service(
        &self,
        new_data_service_model: &NewDataServiceModel,
    ) -> Outcome<dataservice::Model>;

    async fn create_main_data_service(
        &self,
        new_data_service_model: &NewDataServiceModel,
    ) -> Outcome<dataservice::Model>;
    async fn delete_data_service_by_id(
        &self,
        data_service_id: &Urn,
    ) -> Outcome<()>;
}
