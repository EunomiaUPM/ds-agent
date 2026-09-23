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

use crate::data::entities::connector_distro_relation;
use ymir::errors::Outcome;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorDistroRelationRepoTrait: Send + Sync {
    async fn create_relation(
        &self,
        tenant_id: &str,
        distro: &str,
        instance: &str,
    ) -> Outcome<connector_distro_relation::Model>;

    async fn update_relation(
        &self,
        tenant_id: &str,
        distro: &str,
        instance: &str,
    ) -> Outcome<connector_distro_relation::Model>;

    async fn get_relation_by_distribution(
        &self,
        tenant_id: Option<String>,
        distro: &str,
    ) -> Outcome<Option<connector_distro_relation::Model>>;

    async fn get_relation_by_instance(
        &self,
        tenant_id: &str,
        instance: &str,
    ) -> Outcome<Option<connector_distro_relation::Model>>;

    async fn delete_relation_by_distribution(&self, tenant_id: &str, distro: &str) -> Outcome<()>;

    async fn delete_relation_by_instance(
        &self,
        tenant_id: Option<String>,
        instance: &str,
    ) -> Outcome<()>;
}
