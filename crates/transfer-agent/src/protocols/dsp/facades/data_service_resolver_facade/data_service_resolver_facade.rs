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

use crate::protocols::dsp::facades::data_service_resolver_facade::DataServiceFacadeTrait;
use catalog_agent::{DatasetDto, DistributionDto};
use common::config::services::traits::TransferConfigTrait;
use common::config::services::TransferConfig;
use common::config::types::traits::MinKnownConfigTrait;
use common::http_client::HttpClient;
use common::utils::get_urn_from_string;
use connector::{ConnectorInstanceDto, ConnectorInstanceTrait};
use negotiation_agent::AgreementDto;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};

pub struct DataServiceFacadeServiceForDSProtocol {
    config: Arc<TransferConfig>,
    client: Arc<HttpClient>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
}

impl DataServiceFacadeServiceForDSProtocol {
    pub fn new(
        config: Arc<TransferConfig>,
        client: Arc<HttpClient>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
    ) -> Self {
        Self {
            config,
            client,
            connector_entity,
        }
    }
}

#[async_trait::async_trait]
impl DataServiceFacadeTrait for DataServiceFacadeServiceForDSProtocol {
    async fn resolve_connector_by_agreement_id(
        &self,
        agreement_id: &Urn,
        formats: Option<&String>,
    ) -> Outcome<ConnectorInstanceDto> {
        let contracts_url = self.config.contracts().get_host(HostType::Http);
        let catalog_url = self.config.catalog().get_host(HostType::Http);
        let agreement_url = format!(
            "{}/api/v1/negotiation-agent/agreements/{}",
            contracts_url, agreement_id
        );

        // 1. resolve agreement - get target (dataset id)
        let agreement = self
            .client
            .get_json::<AgreementDto>(agreement_url.as_str())
            .await?;
        let agreement_target = get_urn_from_string(&agreement.inner.target)?;

        // 2. resolve dataset entity
        let datasets_url = format!(
            "{}/api/v1/catalog-agent/datasets/{}",
            catalog_url,
            agreement_target.clone()
        );
        let dataset = self
            .client
            .get_json::<DatasetDto>(datasets_url.as_str())
            .await?;
        let dataset_id = get_urn_from_string(&dataset.inner.id)?;

        // 3. resolve distribution by dataset + format
        let distribution_url = format!(
            "{}/api/v1/catalog-agent/distributions/dataset/{}/format/{}",
            catalog_url,
            dataset_id.clone(),
            formats
                .ok_or_else(|| Errors::crazy("dct_formats is required", None))?
                .to_string()
        );
        let distribution = self
            .client
            .get_json::<DistributionDto>(distribution_url.as_str())
            .await?;
        let distribution_id = Urn::from_str(distribution.inner.id.as_str())?;

        // 4. resolve connector instance by distribution
        let connector_instance = self
            .connector_entity
            .get_instance_by_distribution(&distribution_id)
            .await?
            .ok_or_else(|| {
                Errors::crazy(
                    format!(
                        "No connector instance found for distribution {}",
                        distribution_id
                    ),
                    None,
                )
            })?;

        Ok(connector_instance)
    }
}
