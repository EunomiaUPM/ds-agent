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

//! DatasetOfferingService over mocked catalog, data service, dataset, distribution and policy
//! services: defaults from the tenant's main entities, the optional policy and the rollback.

use std::sync::Arc;

use catalog_agent::entities::dataset_offerings::{
    DatasetOfferingInput, DistributionOfferingInput, NewDatasetOfferingDto, PolicyOfferingInput,
};
use catalog_agent::entities::odrl_policies::CatalogEntityTypes;
use catalog_agent::services::catalogs::MockCatalogServiceTrait;
use catalog_agent::services::data_services::MockDataServiceServiceTrait;
use catalog_agent::services::dataset_offerings::service::DatasetOfferingService;
use catalog_agent::services::dataset_offerings::DatasetOfferingServiceTrait;
use catalog_agent::services::datasets::MockDatasetServiceTrait;
use catalog_agent::services::distributions::MockDistributionServiceTrait;
use catalog_agent::services::odrl_policies::MockOdrlPolicyServiceTrait;
use common::test_utils::scopes::TestUsers;
use serde_json::json;
use ymir::errors::Errors;

use crate::support::builders::{
    catalog_dto, data_service_dto, dataset_dto, distribution_dto, odrl_policy_dto,
};
use crate::support::fixtures::{test_urn, urn};

#[derive(Default)]
struct Deps {
    catalogs: MockCatalogServiceTrait,
    data_services: MockDataServiceServiceTrait,
    datasets: MockDatasetServiceTrait,
    distributions: MockDistributionServiceTrait,
    policies: MockOdrlPolicyServiceTrait,
}

impl Deps {
    fn service(self) -> DatasetOfferingService {
        DatasetOfferingService::new(
            Arc::new(self.catalogs),
            Arc::new(self.data_services),
            Arc::new(self.datasets),
            Arc::new(self.distributions),
            Arc::new(self.policies),
        )
    }

    /// Creating the dataset returns dataset 100.
    fn creating_dataset(mut self) -> Self {
        self.datasets
            .expect_create_dataset()
            .returning(|_, _| Ok(dataset_dto(100)));
        self
    }
}

fn offering(
    catalog_id: Option<&str>,
    access_service_id: Option<&str>,
    policy: Option<PolicyOfferingInput>,
) -> NewDatasetOfferingDto {
    NewDatasetOfferingDto {
        dataset: DatasetOfferingInput {
            title: "Weather".to_string(),
            description: None,
            conforms_to: None,
            creator: None,
            catalog_id: catalog_id.map(str::to_string),
        },
        distribution: DistributionOfferingInput {
            title: "Weather API".to_string(),
            description: None,
            formats: None,
            access_service_id: access_service_id.map(str::to_string),
        },
        policy,
    }
}

/// Without ids the dataset goes to the main catalog and the distribution to the main data
/// service, with the default conformance and format.
#[tokio::test]
async fn defaults_to_the_main_catalog_and_data_service() {
    let mut deps = Deps::default();
    deps.catalogs
        .expect_get_main_catalog()
        .returning(|_| Ok(Some(catalog_dto(1))));
    deps.datasets
        .expect_create_dataset()
        .withf(|_, d| {
            d.catalog_id == test_urn(1)
                && d.dct_title.as_deref() == Some("Weather")
                && d.dct_conforms_to.as_deref() == Some("https://w3id.org/dspace/v0.8/dcat")
        })
        .times(1)
        .returning(|_, _| Ok(dataset_dto(100)));
    deps.data_services
        .expect_get_main_data_service()
        .returning(|_| Ok(Some(data_service_dto(200))));
    deps.distributions
        .expect_create_distribution()
        .withf(|_, d| {
            d.dcat_access_service == urn(200)
                && d.dataset_id == test_urn(100)
                && d.dct_formats.as_deref() == Some("application/json")
        })
        .times(1)
        .returning(|_, _| Ok(distribution_dto(300)));

    let created = deps
        .service()
        .create_offering(&TestUsers::user("tenant-1", "/admin/tenant-1"), &offering(None, None, None))
        .await
        .unwrap();

    assert_eq!(created.dataset.inner.id, urn(100));
    assert_eq!(created.distribution.inner.id, urn(300));
    assert!(created.policy.is_none());
}

/// Explicit catalog and access service ids skip the main-entity lookups.
#[tokio::test]
async fn explicit_ids_skip_the_main_lookups() {
    let mut deps = Deps::default();
    deps.datasets
        .expect_create_dataset()
        .withf(|_, d| d.catalog_id == test_urn(5))
        .returning(|_, _| Ok(dataset_dto(100)));
    deps.distributions
        .expect_create_distribution()
        .withf(|_, d| d.dcat_access_service == "urn:svc:explicit")
        .returning(|_, _| Ok(distribution_dto(300)));

    let result = deps
        .service()
        .create_offering(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &offering(Some(&urn(5)), Some("urn:svc:explicit"), None),
        )
        .await;
    assert!(result.is_ok());
}

/// A tenant without main catalog cannot publish without naming one; nothing is created.
#[tokio::test]
async fn missing_main_catalog_is_not_found() {
    let mut deps = Deps::default();
    deps.catalogs
        .expect_get_main_catalog()
        .returning(|_| Ok(None));

    let result = deps
        .service()
        .create_offering(&TestUsers::user("tenant-1", "/admin/tenant-1"), &offering(None, None, None))
        .await;
    assert!(result.is_err());
}

/// A malformed catalog id is rejected before anything is created.
#[tokio::test]
async fn malformed_catalog_id_is_rejected() {
    let result = Deps::default()
        .service()
        .create_offering(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &offering(Some("not a urn"), None, None),
        )
        .await;
    assert!(result.is_err());
}

/// A policy becomes a single-permission ODRL offer on the dataset, with the default action.
#[tokio::test]
async fn policy_becomes_an_offer_on_the_dataset() {
    let mut deps = Deps::default().creating_dataset();
    deps.distributions
        .expect_create_distribution()
        .returning(|_, _| Ok(distribution_dto(300)));
    deps.policies
        .expect_create_odrl_offer()
        .withf(|_, p| {
            let offer = serde_json::to_value(&p.odrl_offer).unwrap();
            p.entity_id == test_urn(100)
                && p.entity_type == CatalogEntityTypes::Dataset
                && p.description.as_deref() == Some("Dataset Usage Policy")
                && offer["permission"][0]["action"] == "http://www.w3.org/ns/odrl/2/use"
                && offer["permission"][0]["constraint"][0]["leftOperand"] == "dateTime"
        })
        .times(1)
        .returning(|_, _| Ok(odrl_policy_dto(400, "Dataset")));
    let policy = PolicyOfferingInput {
        description: None,
        action: None,
        profile: None,
        constraints: Some(vec![json!({
            "leftOperand": "dateTime",
            "operator": "lteq",
            "rightOperand": "2026-12-31T23:59:59Z"
        })]),
    };

    let created = deps
        .service()
        .create_offering(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &offering(Some(&urn(1)), Some("urn:svc:x"), Some(policy)),
        )
        .await
        .unwrap();
    assert_eq!(created.policy.unwrap().inner.id, urn(400));
}

/// If the distribution fails after the dataset was created, the dataset is removed again.
#[tokio::test]
async fn failure_after_the_dataset_removes_it() {
    let mut deps = Deps::default().creating_dataset();
    deps.distributions
        .expect_create_distribution()
        .returning(|_, _| Err(Errors::crazy("db down", None)));
    deps.datasets
        .expect_delete_dataset_by_id()
        .withf(|_, id| *id == test_urn(100))
        .times(1)
        .returning(|_, _| Ok(()));

    let result = deps
        .service()
        .create_offering(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &offering(Some(&urn(1)), Some("urn:svc:x"), None),
        )
        .await;
    assert!(result.is_err());
}
