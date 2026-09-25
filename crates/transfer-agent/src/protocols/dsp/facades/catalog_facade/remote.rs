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

use catalog_agent::{DatasetDto, DistributionDto};
use common::auth::ServiceHttpClient;
use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use connector::{
    ConnectorInstanceDto, ConnectorInstanceFacadeTrait, ConnectorInstanceRemoteFacade,
};
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use crate::protocols::dsp::facades::catalog_facade::CatalogFacadeTrait;

/// Catalog entities and connector instances read from the catalog agent's API.
pub struct CatalogRemoteFacade {
    catalog_url: String,
    connector: ConnectorInstanceRemoteFacade,
    service_client: Arc<ServiceHttpClient>,
}

impl CatalogRemoteFacade {
    pub fn new(catalog: &MinKnownConfig, service_client: Arc<ServiceHttpClient>) -> Self {
        Self {
            catalog_url: format!(
                "{}{}/{}",
                catalog.get_host(HostType::Http),
                catalog.get_api_version(),
                catalog_agent::SERVICE_NAME
            ),
            connector: ConnectorInstanceRemoteFacade::new(catalog, service_client.clone()),
            service_client,
        }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogRemoteFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog", tenant = %tenant_id)
    )]
    async fn get_dataset(&self, tenant_id: &str, dataset_id: &Urn) -> Outcome<DatasetDto> {
        let url = format!("{}/datasets/{dataset_id}", self.catalog_url);
        self.service_client.get_json(&url, Some(tenant_id)).await
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
        let url = format!(
            "{}/distributions/dataset/{dataset_id}/format/{dct_format}",
            self.catalog_url
        );
        self.service_client.get_json(&url, Some(tenant_id)).await
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
