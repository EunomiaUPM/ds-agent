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

//! PolicyTemplateService with a mocked repository: tenant isolation and role checks.

use std::collections::HashMap;
use std::sync::Arc;

use catalog_agent::data::entities::policy_template;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, PolicyTemplatesRepoErrors,
};
use catalog_agent::data::repo_traits::policy_template_repo::MockPolicyTemplatesRepositoryTrait;
use catalog_agent::entities::filters::PolicyTemplateFilter;
use catalog_agent::entities::policy_templates::NewPolicyTemplateDto;
use catalog_agent::services::policy_templates::service::PolicyTemplateService;
use catalog_agent::services::policy_templates::PolicyTemplateServiceTrait;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use serde_json::json;
use ymir::errors::RepoIntoErrors;

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::PolicyTemplatesRepoErrors(
        PolicyTemplatesRepoErrors::PolicyTemplateNotFound,
    )
    .into_errors()
}

fn make_svc(repo: MockPolicyTemplatesRepositoryTrait) -> PolicyTemplateService {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_policy_template_repo()
        .returning(move || repo.clone());
    PolicyTemplateService::new(Arc::new(factory))
}

fn make_model(tenant: &str, id: &str, version: &str) -> policy_template::Model {
    policy_template::Model {
        user_id: tenant.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
        id: id.to_string(),
        version: version.to_string(),
        date: Utc::now().into(),
        author: "tester".to_string(),
        title: None,
        description: None,
        content: json!({}),
        parameters: json!({}),
    }
}

fn make_new_dto() -> NewPolicyTemplateDto {
    NewPolicyTemplateDto {
        id: Some("tpl-1".to_string()),
        visibility: None,
        owner: None,
        version: Some("1.0".to_string()),
        date: None,
        title: None,
        description: None,
        author: Some("tester".to_string()),
        content: serde_json::from_value(json!({})).unwrap(),
        parameters: HashMap::new(),
    }
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_policy_template_by_id_and_version()
        .withf(|scope, id, version| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == "tpl-1" && version == "1.0")
        .returning(|_, _, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_policies_template_by_version_and_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), "tpl-1", "1.0")
        .await
        .is_err());
}

/// The versions of a template are looked up in the caller's tenant.
#[tokio::test]
async fn get_versions_by_id_is_tenant_scoped() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_policy_templates_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == "tpl-1")
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let dtos = svc
        .get_policies_template_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), "tpl-1")
        .await
        .unwrap();
    assert!(dtos.is_empty());
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_all_policy_templates()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_svc(repo);
    let filter = PolicyTemplateFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    let page = svc
        .get_all_policy_templates(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default()
        )
        .await
        .unwrap();
    assert!(page.items.is_empty());
}

/// An admin without a tenant filter lists every tenant.
#[tokio::test]
async fn get_all_admin_without_tenant_queries_cross_tenant() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_all_policy_templates()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| {
            Ok((
                vec![
                    make_model("tenant-1", "tpl-1", "1.0"),
                    make_model("tenant-2", "tpl-2", "1.0"),
                ],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_policy_templates(
            &TestUsers::user("admin-tenant", "/admin"),
            &PolicyTemplateFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_delete_policy_template_by_id_and_version()
        .withf(|scope, id, version| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == "tpl-1" && version == "1.0")
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_policy_template_by_version_and_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), "tpl-1", "1.0")
        .await
        .is_err());
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_batch_policy_templates()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == ["tpl-1".to_string()])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_policy_templates(&TestUsers::user("tenant-2", "/admin/tenant-2"), &["tpl-1".to_string()])
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_create_policy_template()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, "tpl-1", "1.0")));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-1"));
    let dto = svc
        .create_policy_template(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.user_id, "tenant-2");
}

/// An admin creates in the tenant named by the DTO.
#[tokio::test]
async fn create_admin_respects_requested_tenant() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_create_policy_template()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-9"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, "tpl-1", "1.0")));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-9"));
    let dto = svc
        .create_policy_template(&TestUsers::user("admin-tenant", "/admin"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.user_id, "tenant-9");
}
