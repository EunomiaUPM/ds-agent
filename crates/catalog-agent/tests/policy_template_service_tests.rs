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

//! Multi-tenant isolation tests for PolicyTemplateService with a mocked repository.

mod fixtures;

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
use fixtures::{admin_scope, reader_scope, tenant_scope};
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
        tenant_id: tenant.to_string(),
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
        tenant_id: None,
        version: Some("1.0".to_string()),
        date: None,
        title: None,
        description: None,
        author: Some("tester".to_string()),
        content: serde_json::from_value(json!({})).unwrap(),
        parameters: HashMap::new(),
    }
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_policy_template_by_id_and_version()
        .withf(|tenant, id, version| tenant == "tenant-2" && id == "tpl-1" && version == "1.0")
        .returning(|_, _, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_policies_template_by_version_and_id(&tenant_scope("tenant-2"), "tpl-1", "1.0")
        .await
        .is_err());
}

#[tokio::test]
async fn get_versions_by_id_is_tenant_scoped() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_policy_templates_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == "tpl-1")
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let dtos = svc
        .get_policies_template_by_id(&tenant_scope("tenant-2"), "tpl-1")
        .await
        .unwrap();
    assert!(dtos.is_empty());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_svc(MockPolicyTemplatesRepositoryTrait::new());
    let filter = PolicyTemplateFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    assert!(svc
        .get_all_policy_templates(
            &tenant_scope("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default()
        )
        .await
        .is_err());
}

#[tokio::test]
async fn get_all_admin_without_tenant_queries_cross_tenant() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_all_policy_templates()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| {
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
            &admin_scope(),
            &PolicyTemplateFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_delete_policy_template_by_id_and_version()
        .withf(|tenant, id, version| tenant == "tenant-2" && id == "tpl-1" && version == "1.0")
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_policy_template_by_version_and_id(&tenant_scope("tenant-2"), "tpl-1", "1.0")
        .await
        .is_err());
}

#[tokio::test]
async fn delete_reader_is_forbidden_before_reaching_repo() {
    let svc = make_svc(MockPolicyTemplatesRepositoryTrait::new());
    assert!(svc
        .delete_policy_template_by_version_and_id(&reader_scope("tenant-1"), "tpl-1", "1.0")
        .await
        .is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_get_batch_policy_templates()
        .withf(|tenant, ids| tenant == "tenant-2" && ids == ["tpl-1".to_string()])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_policy_templates(&tenant_scope("tenant-2"), &["tpl-1".to_string()])
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_create_policy_template()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| Ok(make_model(&cmd.tenant_id, "tpl-1", "1.0")));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.tenant_id = Some("tenant-1".to_string());
    let dto = svc
        .create_policy_template(&tenant_scope("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.tenant_id, "tenant-2");
}

#[tokio::test]
async fn create_admin_respects_requested_tenant() {
    let mut repo = MockPolicyTemplatesRepositoryTrait::new();
    repo.expect_create_policy_template()
        .withf(|cmd| cmd.tenant_id == "tenant-9")
        .returning(|cmd| Ok(make_model(&cmd.tenant_id, "tpl-1", "1.0")));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.tenant_id = Some("tenant-9".to_string());
    let dto = svc
        .create_policy_template(&admin_scope(), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.tenant_id, "tenant-9");
}

#[tokio::test]
async fn create_reader_is_forbidden() {
    let svc = make_svc(MockPolicyTemplatesRepositoryTrait::new());
    assert!(svc
        .create_policy_template(&reader_scope("tenant-1"), &make_new_dto())
        .await
        .is_err());
}
