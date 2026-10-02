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

//! ParameterStoreImpl with a mocked repository: tenant isolation and role checks.

use std::sync::Arc;

use common::test_utils::scopes::TestScopes;
use keystore::data::repo::parameters::{MockParameterRepoTrait, ParameterRepoErrors};
use keystore::entities::commands::{EditParameterCommand, NewParameterCommand};
use keystore::entities::entry::Entry;
use keystore::entities::filters::PrefixFilter;
use keystore::entities::key::Key;
use keystore::entities::version::Version;
use keystore::services::parameters::ParameterStore;
use keystore::services::parameters::service::ParameterStoreImpl;
use ymir::errors::RepoIntoErrors;

use crate::support::fixtures::metadata;

fn test_key(suffix: &str) -> Key {
    Key::new(format!("/config/param-{suffix}")).unwrap()
}

fn make_service(repo: MockParameterRepoTrait) -> ParameterStoreImpl {
    ParameterStoreImpl::new(Arc::new(repo))
}

fn make_entry(tenant: &str, key: Key, value: serde_json::Value) -> Entry<serde_json::Value> {
    Entry {
        metadata: metadata(tenant, key),
        value,
    }
}

fn make_new_cmd(key: Key, tenant_id: Option<String>) -> NewParameterCommand<serde_json::Value> {
    NewParameterCommand {
        key,
        value: serde_json::json!({"test": true}),
        description: Some("test description".to_string()),
        tenant_id,
    }
}

fn make_edit_cmd() -> EditParameterCommand<serde_json::Value> {
    EditParameterCommand {
        value: serde_json::json!({"updated": true}),
        expected_version: Version::INITIAL,
        description: Some("updated description".to_string()),
    }
}

/// Reading a key of another tenant is not found: the lookup only searches the caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockParameterRepoTrait::new();
    let key = test_key("1");
    repo.expect_get_parameter_by_key()
        .withf(move |tenant, k| tenant == "tenant-2" && k == &test_key("1"))
        .returning(|_, _| Ok(None));

    let svc = make_service(repo);
    assert!(
        svc.read(&TestScopes::owner("tenant-2"), &key)
            .await
            .is_err()
    );
}

/// A non-admin listing another tenant is rejected before touching the repository.
#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockParameterRepoTrait::new();
    let svc = make_service(repo);

    let filter = PrefixFilter {
        prefix: None,
        tenant_id: Some("tenant-foreign".to_string()),
    };

    let result = svc.list(&TestScopes::owner("tenant-1"), &filter).await;
    assert!(result.is_err());
}

/// Updating a key of another tenant is not found and changes nothing.
#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut repo = MockParameterRepoTrait::new();
    let key = test_key("1");
    repo.expect_put_parameter()
        .withf(move |tenant, k, _| tenant == "tenant-2" && k == &test_key("1"))
        .returning(|_, _, _| Err(ParameterRepoErrors::ParameterNotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.update(
            &TestScopes::owner("tenant-2"),
            &key,
            &make_edit_cmd(),
            "tester"
        )
        .await
        .is_err()
    );
}

/// Deleting a key of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockParameterRepoTrait::new();
    let key = test_key("1");
    repo.expect_delete_parameter()
        .withf(move |tenant, k| tenant == "tenant-2" && k == &test_key("1"))
        .returning(|_, _| Err(ParameterRepoErrors::ParameterNotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.delete(&TestScopes::owner("tenant-2"), &key)
            .await
            .is_err()
    );
}

/// A batch read only returns entries of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockParameterRepoTrait::new();
    let key = test_key("1");
    repo.expect_get_batch_parameters()
        .withf(move |tenant, keys| tenant == "tenant-2" && *keys == [test_key("1")])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(repo);
    let entries = svc
        .batch(&TestScopes::owner("tenant-2"), &[key])
        .await
        .unwrap();
    assert!(entries.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the command says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockParameterRepoTrait::new();
    let key = test_key("1");
    repo.expect_create_parameter()
        .withf(|tenant, cmd| tenant == "tenant-2" && cmd.tenant_id.as_deref() == Some("tenant-2"))
        .returning(|tenant, cmd| Ok(make_entry(tenant, cmd.key.clone(), cmd.value.clone())));

    let svc = make_service(repo);
    let cmd = make_new_cmd(key.clone(), Some("tenant-1".to_string()));
    let entry = svc
        .create(&TestScopes::owner("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(entry.metadata.tenant_id, "tenant-2");
}

/// A reader cannot create a parameter.
#[tokio::test]
async fn reader_cannot_create_parameter() {
    let repo = MockParameterRepoTrait::new();
    let svc = make_service(repo);
    let cmd = make_new_cmd(test_key("1"), None);

    let result = svc.create(&TestScopes::reader("tenant-1"), &cmd).await;
    assert!(result.is_err());
}

/// A reader cannot delete a parameter.
#[tokio::test]
async fn reader_cannot_delete_parameter() {
    let repo = MockParameterRepoTrait::new();
    let svc = make_service(repo);

    let result = svc
        .delete(&TestScopes::reader("tenant-1"), &test_key("1"))
        .await;
    assert!(result.is_err());
}

/// An admin lists without a tenant filter.
#[tokio::test]
async fn admin_can_query_cross_tenant() {
    let mut repo = MockParameterRepoTrait::new();
    repo.expect_get_all_parameters()
        .withf(|f| f.tenant_id.is_none())
        .returning(|_| Ok(vec![]));

    let svc = make_service(repo);
    let result = svc
        .list(&TestScopes::admin(), &PrefixFilter::default())
        .await;
    assert!(result.is_ok());
}

/// An admin creates in the tenant named by the command.
#[tokio::test]
async fn admin_can_create_parameter_for_any_tenant() {
    let mut repo = MockParameterRepoTrait::new();
    let key = test_key("1");
    repo.expect_create_parameter()
        .withf(|tenant, cmd| {
            tenant == "tenant-custom" && cmd.tenant_id.as_deref() == Some("tenant-custom")
        })
        .returning(|tenant, cmd| Ok(make_entry(tenant, cmd.key.clone(), cmd.value.clone())));

    let svc = make_service(repo);
    let cmd = make_new_cmd(key, Some("tenant-custom".to_string()));
    let entry = svc.create(&TestScopes::admin(), &cmd).await.unwrap();
    assert_eq!(entry.metadata.tenant_id, "tenant-custom");
}
