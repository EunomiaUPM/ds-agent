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

//! OrchestrationPersistenceForProtocol building the DSP catalog from mocked entity services.

use std::sync::Arc;

use catalog_agent::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use catalog_agent::protocols::dsp::types::catalog_definition::CatalogCatalogTypes;
use catalog_agent::services::catalogs::MockCatalogServiceTrait;
use catalog_agent::services::data_services::MockDataServiceServiceTrait;
use catalog_agent::services::datasets::MockDatasetServiceTrait;
use catalog_agent::services::distributions::MockDistributionServiceTrait;
use catalog_agent::services::odrl_policies::MockOdrlPolicyServiceTrait;
use common::paginated_spec::{Paginated, MAX_PAGE_LIMIT};
use common::test_utils::grpc::TENANT;
use common::test_utils::scopes::TestScopes;
use mockall::Sequence;

use crate::support::builders::{catalog_dto, data_service_dto};

/// Every sub-catalog reaches the DSP answer, following the cursor past the first page.
#[tokio::test]
async fn catalog_lists_sub_catalogs_from_every_page() {
    let mut catalogs = MockCatalogServiceTrait::new();
    catalogs
        .expect_get_main_catalog()
        .returning(|_| Ok(Some(catalog_dto(1))));
    let mut seq = Sequence::new();
    catalogs
        .expect_get_all_catalogs()
        .withf(|_, filter, page, _| {
            filter.with_main_catalog == Some(false)
                && page.limit == MAX_PAGE_LIMIT
                && page.cursor.is_none()
        })
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_, _, _, _| {
            let items = (2..=101).map(catalog_dto).collect();
            Ok(Paginated::new(items, Some("next".to_string()), Some(120)))
        });
    catalogs
        .expect_get_all_catalogs()
        .withf(|_, _, page, _| page.cursor.as_deref() == Some("next"))
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_, _, _, _| {
            let items = (102..=121).map(catalog_dto).collect();
            Ok(Paginated::new(items, None, Some(120)))
        });

    let mut data_services = MockDataServiceServiceTrait::new();
    data_services
        .expect_get_main_data_service()
        .returning(|_| Ok(Some(data_service_dto(50))));
    data_services
        .expect_get_data_services_by_catalog_id()
        .times(120)
        .returning(|_, _| Ok(vec![]));

    let mut datasets = MockDatasetServiceTrait::new();
    datasets
        .expect_get_datasets_by_catalog_id()
        .times(121)
        .returning(|_, _| Ok(vec![]));

    let persistence = OrchestrationPersistenceForProtocol::new(
        Arc::new(catalogs),
        Arc::new(data_services),
        Arc::new(datasets),
        Arc::new(MockOdrlPolicyServiceTrait::new()),
        Arc::new(MockDistributionServiceTrait::new()),
    );
    let catalog = persistence
        .get_catalog(&TestScopes::reader(TENANT))
        .await
        .unwrap();

    match catalog.catalogs {
        CatalogCatalogTypes::CatalogMultipleMinimized(subs) => assert_eq!(subs.len(), 120),
        CatalogCatalogTypes::CatalogMultipleOriginal(_) => {
            panic!("expected minimized sub-catalogs")
        }
    }
}
