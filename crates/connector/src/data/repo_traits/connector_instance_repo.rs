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

use crate::data::entities::connector_instances;
use crate::data::entities::connector_instances::NewConnectorInstanceModel;
use ymir::errors::Outcome;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorInstanceRepoTrait: Send + Sync {
    async fn create_instance(
        &self,
        new_instance_model: &NewConnectorInstanceModel,
    ) -> Outcome<connector_instances::Model>;

    async fn get_instance_by_id(
        &self,
        instance_id: &String,
    ) -> Outcome<Option<connector_instances::Model>>;

    async fn get_instance_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<Option<connector_instances::Model>>;

    async fn get_instances_by_distribution(
        &self,
        distribution_id: &String,
    ) -> Outcome<Option<connector_instances::Model>>;

    async fn delete_instance_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<()>;

    async fn delete_instance_by_id(&self, instance_id: &String) -> Outcome<()>;
}
