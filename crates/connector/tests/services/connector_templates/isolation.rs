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

//! Owner isolation on template reads and writes.

use super::*;

fn tenant_2() -> UserInfo {
    TestUsers::user("tenant-2", "/admin/tenant-2")
}

/// Reading a template the caller does not see finds nothing.
#[tokio::test]
async fn get_one_unseen_returns_none() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_template_by_name_and_version()
        .withf(|scope, name, ver| {
            *scope == OwnerScope::seeing(&tenant_2()) && name == "my-template" && ver == "1.0.0"
        })
        .times(1)
        .returning(|_, _, _| Ok(None));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let res = svc
        .get_template_by_name_and_version(&tenant_2(), "my-template", "1.0.0")
        .await
        .unwrap();

    assert!(res.is_none());
}

/// A listing covers what the caller sees, narrowed by the author it asks for.
#[tokio::test]
async fn get_all_covers_what_the_caller_sees() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_all_templates()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::user("tenant-1", "/admin/tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .times(1)
        .returning(|_, _, _, _| Ok((vec![], Some(0))));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let filter = ConnectorTemplateFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    let result = svc
        .get_all_templates(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &filter,
            &Page::default(),
            Sort::CreatedAtDesc,
        )
        .await;

    assert!(result.is_ok());
}

/// A template belongs to whoever creates it, private.
#[tokio::test]
async fn create_belongs_to_the_caller() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_create_template()
        .withf(|m| m.owner == Owner::private(&tenant_2()))
        .times(1)
        .returning(|m| Ok(echo_model(m)));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let mut dto = valid_template_dto();
    let result = svc
        .create_template(&tenant_2(), &mut dto)
        .await;

    assert!(result.is_ok());
}

/// Deleting a template the caller does not act on is not found.
#[tokio::test]
async fn delete_unreached_returns_not_found() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_delete_template_by_name_and_version()
        .withf(|scope, name, ver| {
            *scope == OwnerScope::acting(&tenant_2()) && name == "my-template" && ver == "1.0.0"
        })
        .times(1)
        .returning(|_, _, _| Err(ConnectorTemplateRepoErrors::TemplateNotFound.into_errors()));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let result = svc
        .delete_template_by_name_and_version(&tenant_2(), "my-template", "1.0.0")
        .await;

    assert!(result.is_err());
}

/// The versions of a template are looked up among those the caller sees.
#[tokio::test]
async fn get_templates_by_id_scoped_to_what_the_caller_sees() {
    let mut template_repo = MockConnectorTemplateRepoTrait::new();
    template_repo
        .expect_get_templates_by_name()
        .withf(|scope, id| {
            *scope == OwnerScope::seeing(&TestUsers::user("tenant-1", "/admin/tenant-1"))
                && id == "my-template"
        })
        .times(1)
        .returning(|_, _| Ok(vec![]));

    let mut repo = MockConnectorRepoTrait::new();
    repo.expect_get_templates_repo()
        .return_const(Arc::new(template_repo) as Arc<dyn ConnectorTemplateRepoTrait>);

    let svc = ConnectorTemplateService::new(Arc::new(repo));
    let res = svc
        .get_templates_by_id(&TestUsers::user("tenant-1", "/admin/tenant-1"), "my-template")
        .await
        .unwrap();

    assert!(res.is_empty());
}
