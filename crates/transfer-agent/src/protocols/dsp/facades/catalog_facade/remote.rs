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
use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use connector::{
    ConnectorInstanceDto, ConnectorInstanceFacadeTrait, ConnectorInstanceRemoteFacade,
};
use urn::Urn;
use ymir::config::types::HostType;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;
use ymir::errors::Outcome;

use crate::protocols::dsp::facades::catalog_facade::CatalogFacadeTrait;

/// Catalog entities and connector instances read from the catalog agent's API.
pub struct CatalogRemoteFacade {
    catalog_url: String,
    connector: ConnectorInstanceRemoteFacade,
}

impl CatalogRemoteFacade {
    pub fn new(catalog: &MinKnownConfig) -> Self {
        Self {
            catalog_url: format!(
                "{}{}/{}",
                catalog.get_host(HostType::Http),
                catalog.get_api_version(),
                catalog_agent::SERVICE_NAME
            ),
            connector: ConnectorInstanceRemoteFacade::new(catalog),
        }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogRemoteFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog")
    )]
    async fn get_dataset(&self, dataset_id: &Urn) -> Outcome<DatasetDto> {
        let url = format!("{}/datasets/{dataset_id}", self.catalog_url);
        http_client().get_json(&url, None).await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog")
    )]
    async fn get_distribution_by_format(
        &self,
        dataset_id: &Urn,
        dct_format: &str,
    ) -> Outcome<DistributionDto> {
        let url = format!(
            "{}/distributions/dataset/{dataset_id}/format/{dct_format}",
            self.catalog_url
        );
        http_client().get_json(&url, None).await
    }

    /// Traced by the connector facade itself.
    async fn get_instance_by_distribution(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        self.connector
            .get_instance_by_distribution(distribution_id)
            .await
    }
}
