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

use catalog_agent::{DatasetDto, DistributionDto};
use connector::ConnectorInstanceDto;
use urn::Urn;
use ymir::errors::Outcome;

pub mod local;
pub mod remote;

/// The catalog agent, as transfer needs it: datasets, distributions and the connector
/// instances it hosts.
#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogFacadeTrait: Send + Sync {
    async fn get_dataset(&self, tenant_id: &str, dataset_id: &Urn) -> Outcome<DatasetDto>;

    async fn get_distribution_by_format(
        &self,
        tenant_id: &str,
        dataset_id: &Urn,
        dct_format: &str,
    ) -> Outcome<DistributionDto>;

    async fn get_instance_by_distribution(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>>;
}
