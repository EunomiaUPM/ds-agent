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

use std::sync::Arc;

use catalog_agent::services::datasets::DatasetServiceTrait;
use catalog_agent::services::distributions::DistributionServiceTrait;
use catalog_agent::{DatasetDto, DistributionDto};
use common::auth::AccessScope;
use connector::{ConnectorInstanceDto, ConnectorInstanceFacadeTrait};
use urn::Urn;
use ymir::errors::Outcome;

use crate::protocols::dsp::facades::catalog_facade::CatalogFacadeTrait;

/// Catalog entities and connector instances read in-process, when catalog shares the process.
pub struct CatalogLocalFacade {
    datasets: Arc<dyn DatasetServiceTrait>,
    distributions: Arc<dyn DistributionServiceTrait>,
    connector: Arc<dyn ConnectorInstanceFacadeTrait>,
}

impl CatalogLocalFacade {
    /// `connector` is the catalog's own in-process connector adapter.
    pub fn new(
        datasets: Arc<dyn DatasetServiceTrait>,
        distributions: Arc<dyn DistributionServiceTrait>,
        connector: Arc<dyn ConnectorInstanceFacadeTrait>,
    ) -> Self {
        Self {
            datasets,
            distributions,
            connector,
        }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogLocalFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog", tenant = %tenant_id)
    )]
    async fn get_dataset(&self, tenant_id: &str, dataset_id: &Urn) -> Outcome<DatasetDto> {
        self.datasets
            .get_dataset_by_id(&AccessScope::service(tenant_id), dataset_id)
            .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog", tenant = %tenant_id)
    )]
    async fn get_distribution_by_format(
        &self,
        tenant_id: &str,
        dataset_id: &Urn,
        dct_format: &str,
    ) -> Outcome<DistributionDto> {
        self.distributions
            .get_distribution_by_dataset_id_and_dct_format(
                &AccessScope::service(tenant_id),
                dataset_id,
                dct_format,
            )
            .await
    }

    /// Traced by the connector facade itself.
    async fn get_instance_by_distribution(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        self.connector
            .get_instance_by_distribution(tenant_id, distribution_id)
            .await
    }
}
