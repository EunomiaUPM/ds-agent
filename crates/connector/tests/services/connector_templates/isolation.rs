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

//! Tenant isolation and role checks on template reads and writes.

use super::*;

/// Reading a template of another tenant finds nothing.
#[tokio::test]
async fn get_one_foreign_tenant_returns_none() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_template_by_name_and_version()
        .withf(|tenant, name, ver| tenant == "tenant-2" && name == "my-template" && ver == "1.0.0")
        .times(1)
        .returning(|_, _, _| Ok(None));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let res = svc
        .get_template_by_name_and_version(&TestScopes::owner("tenant-2"), "my-template", "1.0.0")
        .await
        .unwrap();

    assert!(res.is_none());
}

/// A non-admin listing another tenant is rejected before touching the repository.
#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockConnectorRepoTrait::new();
    let svc = ConnectorTemplateService::new(Arc::new(repo));

    let filter = ConnectorTemplateFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .get_all_templates(
            &TestScopes::owner("tenant-1"),
            &filter,
            &Page::default(),
            Sort::CreatedAtDesc,
        )
        .await;

    assert!(result.is_err());
}

/// A non-admin listing without filter only sees its own tenant.
#[tokio::test]
async fn get_all_filters_by_caller_tenant_for_non_admin() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_all_templates()
        .withf(|f, _, _| f.tenant_id.as_deref() == Some("tenant-1"))
        .times(1)
        .returning(|_, _, _| Ok((vec![], Some(0))));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let filter = ConnectorTemplateFilter::default();
    let result = svc
        .get_all_templates(
            &TestScopes::owner("tenant-1"),
            &filter,
            &Page::default(),
            Sort::CreatedAtDesc,
        )
        .await;

    assert!(result.is_ok());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_create_template()
        .withf(|m| m.tenant_id == "tenant-2")
        .times(1)
        .returning(|m| Ok(echo_model(m)));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let mut dto = valid_template_dto();
    let result = svc
        .create_template(&TestScopes::owner("tenant-2"), &mut dto)
        .await;

    assert!(result.is_ok());
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_delete_template_by_name_and_version()
        .withf(|tenant, name, ver| tenant == "tenant-2" && name == "my-template" && ver == "1.0.0")
        .times(1)
        .returning(|_, _, _| Err(ConnectorTemplateRepoErrors::TemplateNotFound.into_errors()));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let result = svc
        .delete_template_by_name_and_version(&TestScopes::owner("tenant-2"), "my-template", "1.0.0")
        .await;

    assert!(result.is_err());
}

/// A reader can neither create nor delete a template.
#[tokio::test]
async fn reader_cannot_create_or_delete() {
    let repo = MockConnectorRepoTrait::new();
    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let reader = TestScopes::reader("tenant-1");

    let mut dto = valid_template_dto();
    let create_res = svc.create_template(&reader, &mut dto).await;
    assert!(create_res.is_err());

    let delete_res = svc
        .delete_template_by_name_and_version(&reader, "my-template", "1.0.0")
        .await;
    assert!(delete_res.is_err());
}

/// The versions of a template are looked up in the caller's tenant.
#[tokio::test]
async fn get_templates_by_id_scoped_to_acting_tenant() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_templates_by_name()
        .withf(|tenant, id| tenant == "tenant-1" && id == "my-template")
        .times(1)
        .returning(|_, _| Ok(vec![]));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let res = svc
        .get_templates_by_id(&TestScopes::owner("tenant-1"), "my-template")
        .await
        .unwrap();

    assert!(res.is_empty());
}
