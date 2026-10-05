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

//! TenantProvisioningService over mocked catalog and data service services and participant
//! facade: idempotent creation of the connector's main catalog and data service, by the root.

use std::sync::Arc;

use catalog_agent::services::catalogs::MockCatalogServiceTrait;
use catalog_agent::services::data_services::MockDataServiceServiceTrait;
use catalog_agent::services::tenant_provisioning::service::TenantProvisioningService;
use catalog_agent::services::tenant_provisioning::TenantProvisioningServiceTrait;
use common::facades::mates_facade::MockMatesFacadeTrait;
use common::oauth::RoleTrait;
use common::test_utils::scopes::TestUsers;

use crate::support::builders::{catalog_dto, data_service_dto, mate};
use crate::support::fixtures::{test_urn, urn};

const DSP_URL: &str = "https://connector.example/dsp";

#[derive(Default)]
struct Deps {
    catalogs: MockCatalogServiceTrait,
    data_services: MockDataServiceServiceTrait,
    mates: MockMatesFacadeTrait,
}

impl Deps {
    fn service(self) -> TenantProvisioningService {
        TenantProvisioningService::new(
            Arc::new(self.catalogs),
            Arc::new(self.data_services),
            Arc::new(self.mates),
            DSP_URL.to_string(),
        )
    }
}

/// Only the root provisions the connector's main catalog.
#[tokio::test]
async fn only_the_root_provisions() {
    let result = Deps::default()
        .service()
        .provision(&TestUsers::user("tenant-1", "/admin/tenant-1"))
        .await;
    assert!(result.is_err());
}

/// A bare connector gets a main catalog naming its participant, and a main data service on
/// the DSP endpoint inside that catalog, both created by the root.
#[tokio::test]
async fn bare_connector_gets_main_catalog_and_data_service() {
    let mut deps = Deps::default();
    deps.catalogs
        .expect_get_main_catalog()
        .returning(|_| Ok(None));
    deps.mates
        .expect_get_me_mate()
        .returning(|| Ok(mate("did:web:connector")));
    deps.catalogs
        .expect_create_main_catalog()
        .withf(|user, c| {
            user.is_root() && c.dspace_participant_id.as_deref() == Some("did:web:connector")
        })
        .times(1)
        .returning(|_, _| Ok(catalog_dto(1)));
    deps.data_services
        .expect_get_main_data_service()
        .returning(|_| Ok(None));
    deps.data_services
        .expect_create_main_data_service()
        .withf(|user, d| {
            user.is_root()
                && d.dcat_endpoint_url == DSP_URL
                && d.catalog_id == test_urn(1)
        })
        .times(1)
        .returning(|_, _| Ok(data_service_dto(200)));

    let provisioned = deps
        .service()
        .provision(&TestUsers::user("admin-tenant", "/admin"))
        .await
        .unwrap();

    assert_eq!(provisioned.catalog.inner.id, urn(1));
    assert_eq!(provisioned.data_service.inner.id, urn(200));
}

/// Provisioning twice changes nothing: existing main entities are kept.
#[tokio::test]
async fn existing_main_entities_are_kept() {
    let mut deps = Deps::default();
    deps.catalogs
        .expect_get_main_catalog()
        .returning(|_| Ok(Some(catalog_dto(1))));
    deps.data_services
        .expect_get_main_data_service()
        .returning(|_| Ok(Some(data_service_dto(200))));

    let provisioned = deps
        .service()
        .provision(&TestUsers::root())
        .await
        .unwrap();
    assert_eq!(provisioned.catalog.inner.id, urn(1));
}
