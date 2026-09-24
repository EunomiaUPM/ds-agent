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

use std::str::FromStr;
use std::sync::Arc;

use common::auth::AccessScope;
use serde_json::json;
use tracing::warn;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::dataset_offerings::{
    DatasetOfferingDto, NewDatasetOfferingDto, PolicyOfferingInput,
};
use crate::entities::datasets::{DatasetDto, NewDatasetDto};
use crate::entities::distributions::{DistributionDto, NewDistributionDto};
use crate::entities::odrl_policies::{CatalogEntityTypes, NewOdrlPolicyDto, OdrlPolicyDto};
use crate::services::catalogs::CatalogServiceTrait;
use crate::services::data_services::DataServiceServiceTrait;
use crate::services::dataset_offerings::DatasetOfferingServiceTrait;
use crate::services::datasets::DatasetServiceTrait;
use crate::services::distributions::DistributionServiceTrait;
use crate::services::odrl_policies::OdrlPolicyServiceTrait;

const DEFAULT_CONFORMS_TO: &str = "https://w3id.org/dspace/v0.8/dcat";
const DEFAULT_FORMAT: &str = "application/json";
const ODRL_USE: &str = "http://www.w3.org/ns/odrl/2/use";
const ODRL_PROFILE: &str = "http://www.w3.org/ns/odrl/2/";

pub struct DatasetOfferingService {
    catalogs: Arc<dyn CatalogServiceTrait>,
    data_services: Arc<dyn DataServiceServiceTrait>,
    datasets: Arc<dyn DatasetServiceTrait>,
    distributions: Arc<dyn DistributionServiceTrait>,
    policies: Arc<dyn OdrlPolicyServiceTrait>,
}

impl DatasetOfferingService {
    pub fn new(
        catalogs: Arc<dyn CatalogServiceTrait>,
        data_services: Arc<dyn DataServiceServiceTrait>,
        datasets: Arc<dyn DatasetServiceTrait>,
        distributions: Arc<dyn DistributionServiceTrait>,
        policies: Arc<dyn OdrlPolicyServiceTrait>,
    ) -> Self {
        Self {
            catalogs,
            data_services,
            datasets,
            distributions,
            policies,
        }
    }

    async fn resolve_catalog(&self, scope: &AccessScope, requested: Option<&str>) -> Outcome<Urn> {
        if let Some(id) = requested.filter(|id| !id.trim().is_empty()) {
            return Self::parse_urn(id);
        }
        let main = self
            .catalogs
            .get_main_catalog(scope)
            .await?
            .ok_or_else(|| {
                Errors::missing_resource("main", "the tenant has no main catalog", None)
            })?;
        Self::parse_urn(&main.inner.id)
    }

    async fn resolve_access_service(
        &self,
        scope: &AccessScope,
        requested: Option<&str>,
    ) -> Outcome<String> {
        if let Some(id) = requested.filter(|id| !id.trim().is_empty()) {
            return Ok(id.trim().to_string());
        }
        let main = self
            .data_services
            .get_main_data_service(scope)
            .await?
            .ok_or_else(|| {
                Errors::missing_resource("main", "the tenant has no main data service", None)
            })?;
        Ok(main.inner.id)
    }

    async fn create_rest(
        &self,
        scope: &AccessScope,
        offering: &NewDatasetOfferingDto,
        dataset: &DatasetDto,
    ) -> Outcome<(DistributionDto, Option<OdrlPolicyDto>)> {
        let dataset_id = Self::parse_urn(&dataset.inner.id)?;
        let input = &offering.distribution;
        let access_service = self
            .resolve_access_service(scope, input.access_service_id.as_deref())
            .await?;
        let distribution = self
            .distributions
            .create_distribution(
                scope,
                &NewDistributionDto {
                    id: None,
                    tenant_id: Some(dataset.inner.tenant_id.clone()),
                    dct_title: Some(input.title.clone()),
                    dct_description: input.description.clone(),
                    dct_formats: Some(
                        input
                            .formats
                            .clone()
                            .unwrap_or_else(|| DEFAULT_FORMAT.to_string()),
                    ),
                    dcat_access_service: access_service,
                    dataset_id: dataset_id.clone(),
                },
            )
            .await?;

        let policy = match &offering.policy {
            Some(policy) => Some(
                self.policies
                    .create_odrl_offer(
                        scope,
                        &Self::policy_command(policy, &dataset.inner.tenant_id, dataset_id)?,
                    )
                    .await?,
            ),
            None => None,
        };
        Ok((distribution, policy))
    }

    fn policy_command(
        policy: &PolicyOfferingInput,
        tenant_id: &str,
        dataset_id: Urn,
    ) -> Outcome<NewOdrlPolicyDto> {
        let offer = json!({
            "profile": [policy.profile.as_deref().unwrap_or(ODRL_PROFILE)],
            "permission": [{
                "action": policy.action.as_deref().unwrap_or(ODRL_USE),
                "constraint": policy.constraints.clone().unwrap_or_default(),
            }],
        });
        Ok(NewOdrlPolicyDto {
            id: None,
            tenant_id: Some(tenant_id.to_string()),
            odrl_offer: serde_json::from_value(offer)
                .map_err(|e| Errors::format(BadFormat::Received, e.to_string(), None))?,
            entity_id: dataset_id,
            entity_type: CatalogEntityTypes::Dataset,
            source_template_id: None,
            source_template_version: None,
            instantiation_parameters: None,
            description: Some(
                policy
                    .description
                    .clone()
                    .unwrap_or_else(|| "Dataset Usage Policy".to_string()),
            ),
        })
    }

    fn parse_urn(id: &str) -> Outcome<Urn> {
        Urn::from_str(id.trim())
            .map_err(|e| Errors::format(BadFormat::Received, e.to_string(), None))
    }
}

#[async_trait::async_trait]
impl DatasetOfferingServiceTrait for DatasetOfferingService {
    async fn create_offering(
        &self,
        scope: &AccessScope,
        offering: &NewDatasetOfferingDto,
    ) -> Outcome<DatasetOfferingDto> {
        scope.require_write()?;
        let input = &offering.dataset;
        let catalog_id = self
            .resolve_catalog(scope, input.catalog_id.as_deref())
            .await?;
        let dataset = self
            .datasets
            .create_dataset(
                scope,
                &NewDatasetDto {
                    id: None,
                    tenant_id: None,
                    dct_conforms_to: Some(
                        input
                            .conforms_to
                            .clone()
                            .unwrap_or_else(|| DEFAULT_CONFORMS_TO.to_string()),
                    ),
                    dct_creator: input.creator.clone(),
                    dct_title: Some(input.title.clone()),
                    dct_description: input.description.clone(),
                    catalog_id,
                },
            )
            .await?;

        match self.create_rest(scope, offering, &dataset).await {
            Ok((distribution, policy)) => Ok(DatasetOfferingDto {
                dataset,
                distribution,
                policy,
            }),
            Err(e) => {
                let dataset_id = Self::parse_urn(&dataset.inner.id)?;
                if let Err(cleanup) = self.datasets.delete_dataset_by_id(scope, &dataset_id).await {
                    warn!(dataset = %dataset_id, error = %cleanup, "Could not remove partial offering");
                }
                Err(e)
            }
        }
    }
}
