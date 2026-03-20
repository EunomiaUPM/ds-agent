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

use crate::entities::catalogs::{CatalogDto, CatalogEntityTrait};
use crate::entities::data_services::{DataServiceDto, DataServiceEntityTrait};
use crate::entities::datasets::{DatasetDto, DatasetEntityTrait};
use crate::entities::distributions::{DistributionDto, DistributionEntityTrait};
use crate::entities::odrl_policies::{OdrlPolicyDto, OdrlPolicyEntityTrait};
use crate::protocols::dsp::types::catalog_definition::{
    Catalog, CatalogCatalogTypes, CatalogDSpaceDeclaration, CatalogDatasetTypes,
    CatalogDcatDeclaration, CatalogDctDeclaration, CatalogFoafDeclaration, CatalogMinimized,
    CatalogServiceTypes,
};
use crate::protocols::dsp::types::dataservice_definition::{
    DataService, DataServiceDcatDeclaration, DataServiceDctDeclaration,
};
use crate::protocols::dsp::types::dataset_definition::{
    Dataset, DatasetDcatDeclaration, DatasetDctDeclaration, DatasetDistributionTypes,
};
use crate::protocols::dsp::types::distribution_definition::{
    Distribution, DistributionDcatDeclaration, DistributionDctDeclaration,
};
use rainbow_common::dcat_formats::{DctFormats, FormatAction, FormatProtocol};
use rainbow_common::dsp_common::context_field::ContextField;
use rainbow_common::dsp_common::odrl::{OdrlOffer, OdrlPolicyInfo, OdrlTypes};
use rainbow_common::errors::ErrorLog;
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct OrchestrationPersistenceForProtocol {
    pub catalog_entities_service: Arc<dyn CatalogEntityTrait>,
    pub data_service_entities_service: Arc<dyn DataServiceEntityTrait>,
    pub dataset_entities_service: Arc<dyn DatasetEntityTrait>,
    pub odrl_policies_service: Arc<dyn OdrlPolicyEntityTrait>,
    pub distributions_entity_service: Arc<dyn DistributionEntityTrait>,
}

impl OrchestrationPersistenceForProtocol {
    pub fn new(
        catalog_entities_service: Arc<dyn CatalogEntityTrait>,
        data_service_entities_service: Arc<dyn DataServiceEntityTrait>,
        dataset_entities_service: Arc<dyn DatasetEntityTrait>,
        odrl_policies_service: Arc<dyn OdrlPolicyEntityTrait>,
        distributions_entity_service: Arc<dyn DistributionEntityTrait>,
    ) -> Self {
        Self {
            catalog_entities_service,
            data_service_entities_service,
            dataset_entities_service,
            odrl_policies_service,
            distributions_entity_service,
        }
    }

    // =========================================================================
    // Public API
    // =========================================================================

    pub async fn get_catalog(&self) -> Outcome<Catalog> {
        // 1. Main catalog
        let main_catalog_dto = self.fetch_main_catalog_dto().await?;
        let main_catalog_urn = Urn::from_str(&main_catalog_dto.inner.id)?;
        // 1b. Main service
        let main_dataservice_dto = self.fetch_main_dataservice_dto().await?;
        // 2. Sub catalogs
        let sub_catalogs = self.build_sub_catalogs(&main_catalog_urn).await?;
        // 3. Datasets in main catalog
        let datasets = self.build_datasets_for_catalog(&main_catalog_urn).await?;
        // 3b. Dataservice in main catalog
        let main_dataservice = self.map_data_service(main_dataservice_dto);
        // 4. Assembly
        let catalog =
            self.map_main_catalog(main_catalog_dto, main_dataservice, sub_catalogs, datasets);
        Ok(catalog)
    }

    pub async fn get_dataset(&self, dataset_id: &Urn) -> Outcome<Dataset> {
        // 1. fetch dataset
        let dataset_dto = self.fetch_dataset_dto(dataset_id).await?;
        // 2. build policies
        let odrl_offers = self.build_odrl_policies(dataset_id).await?;
        // 3. build distributions
        let distributions = self.build_distributions_with_services(dataset_id).await?;
        // 4. final mapping
        let dataset = self.map_dataset(dataset_dto, odrl_offers, distributions);
        Ok(dataset)
    }

    // =========================================================================
    // Builders
    // =========================================================================
    async fn build_sub_catalogs(&self, exclude_id: &Urn) -> Outcome<Vec<CatalogMinimized>> {
        let catalogs_dtos =
            self.catalog_entities_service.get_all_catalogs(None, None, false).await?;
        let mut dcat_catalogs = Vec::with_capacity(catalogs_dtos.len());
        for catalog_dto in catalogs_dtos {
            let catalog_urn = Urn::from_str(&catalog_dto.inner.id)?;
            if &catalog_urn == exclude_id {
                continue;
            }
            let sub_datasets = self.build_datasets_for_catalog(&catalog_urn).await?;
            let sub_dataservice = self.build_dataservices_for_catalog(&catalog_urn).await?;
            dcat_catalogs.push(self.map_subcatalog(catalog_dto, sub_dataservice, sub_datasets));
        }
        Ok(dcat_catalogs)
    }
    async fn build_datasets_for_catalog(&self, catalog_id: &Urn) -> Outcome<Vec<Dataset>> {
        let datasets_dtos =
            self.dataset_entities_service.get_datasets_by_catalog_id(catalog_id).await?;
        let mut dcat_datasets = Vec::with_capacity(datasets_dtos.len());

        for dataset_dto in datasets_dtos {
            let dataset_urn = Urn::from_str(&dataset_dto.inner.id)?;
            let dcat_dataset = self.get_dataset(&dataset_urn).await?;
            dcat_datasets.push(dcat_dataset);
        }

        Ok(dcat_datasets)
    }

    async fn build_dataservices_for_catalog(&self, catalog_id: &Urn) -> Outcome<Vec<DataService>> {
        let dataservices_dtos =
            self.data_service_entities_service.get_data_services_by_catalog_id(catalog_id).await?;
        let mut dcat_dataservices = Vec::with_capacity(dataservices_dtos.len());

        for dataservices_dto in dataservices_dtos {
            let dcat_dataservice = self.map_data_service(dataservices_dto);
            dcat_dataservices.push(dcat_dataservice);
        }
        Ok(dcat_dataservices)
    }

    async fn build_odrl_policies(&self, entity_id: &Urn) -> Outcome<Vec<OdrlOffer>> {
        let policies_dtos = self
            .odrl_policies_service
            .get_all_odrl_offers_by_entity(entity_id)
            .await
            .unwrap_or_default();
        let offers = policies_dtos
            .into_iter()
            .map(|dto| self.map_odrl_policy(dto))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(offers)
    }

    async fn build_distributions_with_services(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<Vec<Distribution>> {
        let distributions_dtos =
            self.distributions_entity_service.get_distributions_by_dataset_id(dataset_id).await?;
        // batch dataservices
        let access_services_ids: Vec<Urn> = distributions_dtos
            .iter()
            .map(|d| d.inner.dcat_access_service.clone())
            .map(|id_str| Urn::from_str(&id_str))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default();

        let services_batch = self
            .data_service_entities_service
            .get_batch_data_services(&access_services_ids)
            .await?;
        // index indices
        let services_map: HashMap<String, DataService> = services_batch
            .into_iter()
            .map(|dto| (dto.inner.id.clone(), self.map_data_service(dto)))
            .collect();

        // map distributions
        let mut distributions = Vec::with_capacity(distributions_dtos.len());
        for dist_dto in distributions_dtos {
            let service_id = &dist_dto.inner.dcat_access_service;
            let linked_service = services_map.get(service_id).cloned();

            distributions.push(self.map_distribution(dist_dto, linked_service)?);
        }
        Ok(distributions)
    }

    // =========================================================================
    // FETCHERS
    // =========================================================================

    async fn fetch_main_catalog_dto(&self) -> Outcome<CatalogDto> {
        match self.catalog_entities_service.get_main_catalog().await? {
            Some(c) => Ok(c),
            None => {
                let err = Errors::missing_resource("", "Main catalog not found", None);
                Err(err)
            }
        }
    }

    async fn fetch_main_dataservice_dto(&self) -> Outcome<DataServiceDto> {
        match self.data_service_entities_service.get_main_data_service().await? {
            Some(c) => Ok(c),
            None => {
                let err = Errors::missing_resource("", "Main dataservice not found", None);
                Err(err)
            }
        }
    }

    async fn fetch_dataset_dto(&self, dataset_id: &Urn) -> Outcome<DatasetDto> {
        match self.dataset_entities_service.get_dataset_by_id(dataset_id).await? {
            Some(d) => Ok(d),
            None => {
                let err = Errors::missing_resource(dataset_id.as_str(), "Dataset not found", None);
                Err(err)
            }
        }
    }

    // =========================================================================
    // MAPPERS from DTOs to DCAT representations
    // =========================================================================

    fn map_main_catalog(
        &self,
        dto: CatalogDto,
        main_dataservice_dto: DataService,
        catalogs: Vec<CatalogMinimized>,
        datasets: Vec<Dataset>,
    ) -> Catalog {
        Catalog {
            context: ContextField::default(),
            _type: "Catalog".to_string(),
            id: Urn::from_str(&dto.inner.id)
                .unwrap_or_else(|_| Urn::from_str("urn:error").unwrap()),
            foaf: CatalogFoafDeclaration { homepage: dto.inner.foaf_home_page },
            dcat: CatalogDcatDeclaration { theme: None, keyword: None },
            dct: CatalogDctDeclaration {
                conforms_to: dto.inner.dct_conforms_to,
                creator: dto.inner.dct_creator,
                identifier: dto.inner.id.clone(),
                issued: dto.inner.dct_issued.naive_utc(),
                modified: dto.inner.dct_modified.map(|d| d.naive_utc()),
                title: dto.inner.dct_title,
                description: vec![],
            },
            dspace: CatalogDSpaceDeclaration { participant_id: None },
            odrl_offer: None,
            extra_fields: Default::default(),
            catalogs: CatalogCatalogTypes::CatalogMultipleMinimized(catalogs),
            datasets: CatalogDatasetTypes::DatasetMultipleMinimized(
                datasets.iter().map(|d| d.into()).collect(),
            ),
            data_services: CatalogServiceTypes::ServiceMinimized(main_dataservice_dto.into()),
        }
    }

    fn map_catalog(
        &self,
        dto: CatalogDto,
        main_dataservice_dto: Vec<DataService>,
        catalogs: Vec<Catalog>,
        datasets: Vec<Dataset>,
    ) -> Catalog {
        Catalog {
            context: ContextField::default(),
            _type: "Catalog".to_string(),
            id: Urn::from_str(&dto.inner.id)
                .unwrap_or_else(|_| Urn::from_str("urn:error").unwrap()),
            foaf: CatalogFoafDeclaration { homepage: dto.inner.foaf_home_page },
            dcat: CatalogDcatDeclaration { theme: None, keyword: None },
            dct: CatalogDctDeclaration {
                conforms_to: dto.inner.dct_conforms_to,
                creator: dto.inner.dct_creator,
                identifier: dto.inner.id.clone(),
                issued: dto.inner.dct_issued.naive_utc(),
                modified: dto.inner.dct_modified.map(|d| d.naive_utc()),
                title: dto.inner.dct_title,
                description: vec![],
            },
            dspace: CatalogDSpaceDeclaration { participant_id: None },
            odrl_offer: None,
            extra_fields: Default::default(),
            catalogs: CatalogCatalogTypes::CatalogMultipleMinimized(
                catalogs.iter().map(|cat| cat.into()).collect(),
            ),
            datasets: CatalogDatasetTypes::DatasetMultipleMinimized(
                datasets.iter().map(|d| d.into()).collect(),
            ),
            data_services: CatalogServiceTypes::ServiceMultiple(main_dataservice_dto),
        }
    }

    fn map_subcatalog(
        &self,
        dto: CatalogDto,
        main_dataservice_dto: Vec<DataService>,
        datasets: Vec<Dataset>,
    ) -> CatalogMinimized {
        CatalogMinimized {
            _type: "Catalog".to_string(),
            id: Urn::from_str(&dto.inner.id)
                .unwrap_or_else(|_| Urn::from_str("urn:error").unwrap()),
            foaf: CatalogFoafDeclaration { homepage: dto.inner.foaf_home_page },
            dcat: CatalogDcatDeclaration { theme: None, keyword: None },
            dct: CatalogDctDeclaration {
                conforms_to: dto.inner.dct_conforms_to,
                creator: dto.inner.dct_creator,
                identifier: dto.inner.id.clone(),
                issued: dto.inner.dct_issued.naive_utc(),
                modified: dto.inner.dct_modified.map(|d| d.naive_utc()),
                title: dto.inner.dct_title,
                description: vec![],
            },
            dspace: CatalogDSpaceDeclaration { participant_id: None },
            odrl_offer: None,
            extra_fields: Default::default(),
            datasets: CatalogDatasetTypes::DatasetMultipleMinimized(
                datasets.iter().map(|d| d.into()).collect(),
            ),
            data_services: CatalogServiceTypes::ServiceMultiple(main_dataservice_dto),
        }
    }

    fn map_dataset(
        &self,
        dto: DatasetDto,
        policies: Vec<OdrlOffer>,
        distributions: Vec<Distribution>,
    ) -> Dataset {
        Dataset {
            context: ContextField::default(),
            _type: "Dataset".to_string(),
            id: dto.inner.id.clone(),
            dcat: DatasetDcatDeclaration { theme: None, keyword: None },
            dct: DatasetDctDeclaration {
                conforms_to: dto.inner.dct_conforms_to,
                creator: dto.inner.dct_creator,
                identifier: dto.inner.id.clone(),
                issued: dto.inner.dct_issued.naive_utc(),
                modified: dto.inner.dct_modified.map(|d| d.naive_utc()),
                title: dto.inner.dct_title,
                description: vec![],
            },
            odrl_offer: policies,
            extra_fields: Default::default(),
            distribution: DatasetDistributionTypes::DistributionMultipleMinimized(
                distributions.iter().map(|d| d.into()).collect(),
            ),
        }
    }

    fn map_distribution(
        &self,
        dto: DistributionDto,
        service: Option<DataService>,
    ) -> Outcome<Distribution> {
        Ok(Distribution {
            context: ContextField::default(),
            _type: "Distribution".to_string(),
            id: dto.inner.id.clone(),
            dcat: DistributionDcatDeclaration {
                access_service: service.map(|d| CatalogServiceTypes::ServiceMinimized(d.into())),
            },
            dct: DistributionDctDeclaration {
                issued: dto.inner.dct_issued.naive_utc(),
                modified: dto.inner.dct_modified.map(|d| d.naive_utc()),
                title: dto.inner.dct_title,
                description: vec![],
                formats: dto.inner.dct_format.unwrap_or("".to_string()),
            },
            odrl_offer: vec![],
            extra_fields: Default::default(),
        })
    }

    fn map_data_service(&self, dto: DataServiceDto) -> DataService {
        DataService {
            context: ContextField::default(),
            _type: "DataService".to_string(),
            id: dto.inner.id.clone(),
            dcat: DataServiceDcatDeclaration {
                theme: None,
                keyword: None,
                endpoint_description: dto.inner.dcat_endpoint_description,
                endpoint_url: dto.inner.dcat_endpoint_url,
            },
            dct: DataServiceDctDeclaration {
                conforms_to: dto.inner.dct_conforms_to,
                creator: dto.inner.dct_creator,
                identifier: dto.inner.id.clone(),
                issued: dto.inner.dct_issued.naive_utc(),
                modified: dto.inner.dct_modified.map(|d| d.naive_utc()),
                title: dto.inner.dct_title,
                description: vec![],
            },
            odrl_offer: vec![],
            extra_fields: Default::default(),
        }
    }

    fn map_odrl_policy(&self, dto: OdrlPolicyDto) -> Outcome<OdrlOffer> {
        let odrl_info = serde_json::from_value::<OdrlPolicyInfo>(dto.inner.odrl_offer)?;

        Ok(OdrlOffer {
            id: Urn::from_str(&dto.inner.id)?,
            profile: None,
            permission: odrl_info.permission,
            obligation: odrl_info.obligation,
            _type: OdrlTypes::Offer,
            prohibition: odrl_info.prohibition,
            target: None,
            description: dto.inner.description,
        })
    }
}
