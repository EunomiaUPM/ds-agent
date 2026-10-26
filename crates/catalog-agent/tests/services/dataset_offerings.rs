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

//! DatasetOfferingService over mocked repositories: the main catalog and data service fill in
//! what the documents leave out, and a failure removes the dataset again.

use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::dataset_offering::NewDatasetOfferingCommand;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use common::eunomia::EUNOMIA_CATALOG_CONTEXT_URL;
use common::test_utils::scopes::TestUsers;
use serde_json::json;

use crate::support::builders::{data_service, id, main_catalog};
use crate::support::context::TestContext;

fn repo() -> MockResourceRepoTrait {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main().returning(|kind| {
        Ok(Some(match kind {
            ResourceKind::Catalog => main_catalog(1),
            _ => data_service(2, 1),
        }))
    });
    repo
}

/// With no catalog and no access the dataset goes to the main catalog, served by the main
/// data service.
#[tokio::test]
async fn offering_falls_back_to_the_main_resources() {
    let mut repo = repo();
    let datasets = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stored = datasets.clone();
    repo.expect_insert().returning(move |r| {
        stored.lock().unwrap().push(r.clone());
        Ok(r.clone())
    });
    let looked_up = datasets.clone();
    repo.expect_get_by_id().returning(move |_, id| {
        if id.to_string() == id_of_main() {
            return Ok(Some(main_catalog(1)));
        }
        Ok(looked_up.lock().unwrap().iter().find(|r| r.id() == id).cloned())
    });
    repo.expect_get_batch().returning(|_, _| Ok(vec![data_service(2, 1)]));
    let ctx = TestContext::new().resource_repo(repo).build();
    let command: NewDatasetOfferingCommand = serde_json::from_value(json!({
        "dataset": {"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset", "title": {"en": "Readings"}},
        "distribution": {"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Distribution", "format": "http+pull"}
    }))
    .unwrap();

    let offering = ctx
        .offerings()
        .create_offering(&TestUsers::alone("ana"), &command, None)
        .await
        .unwrap();

    assert_eq!(offering.dataset.catalog_id().map(ToString::to_string), Some(id_of_main()));
    assert_eq!(offering.distribution.dataset_id(), Some(offering.dataset.id()));
    assert_eq!(
        offering.distribution.description().first_iri(Vocab::DCAT_ACCESS_SERVICE),
        Some(id(ResourceKind::DataService, 2).as_str())
    );
    assert!(offering.policy.is_none());
}

/// A distribution that is not one makes the offering fail and the dataset go.
#[tokio::test]
async fn failed_offering_removes_its_dataset() {
    let mut repo = repo();
    let created = std::sync::Arc::new(std::sync::Mutex::new(None));
    let kept = created.clone();
    repo.expect_insert().returning(move |r| {
        *kept.lock().unwrap() = Some(r.clone());
        Ok(r.clone())
    });
    let seen = created.clone();
    repo.expect_get_by_id().returning(move |_, id| {
        if id.to_string() == id_of_main() {
            return Ok(Some(main_catalog(1)));
        }
        Ok(seen.lock().unwrap().clone())
    });
    repo.expect_delete().times(1).returning(|_, _| Ok(crate::support::builders::dataset(3, 1)));
    let ctx = TestContext::new().resource_repo(repo).build();
    let command: NewDatasetOfferingCommand = serde_json::from_value(json!({
        "dataset": {"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset", "title": {"en": "Readings"}},
        "distribution": {"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Agent"}
    }))
    .unwrap();

    let failed = ctx.offerings().create_offering(&TestUsers::alone("ana"), &command, None).await;

    assert!(failed.is_err());
}

fn id_of_main() -> String {
    id(ResourceKind::Catalog, 1)
}
