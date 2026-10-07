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

//! SecretStoreImpl with a mocked repository: tenant isolation and role checks.

use std::sync::Arc;

use common::test_utils::scopes::TestUsers;
use keystore::data::repo::secrets::{MockSecretRepoTrait, SecretRepoErrors};
use keystore::entities::commands::{EditSecretCommand, NewSecretCommand};
use keystore::entities::entry::SecretEntry;
use keystore::entities::filters::PrefixFilter;
use keystore::entities::key::Key;
use keystore::entities::secret_value::SecretValue;
use keystore::entities::version::Version;
use keystore::services::secrets::SecretStore;
use keystore::services::secrets::service::SecretStoreImpl;
use ymir::errors::RepoIntoErrors;

use crate::support::fixtures::metadata;

fn test_key(suffix: &str) -> Key {
    Key::new(format!("/secrets/secret-{suffix}")).unwrap()
}

fn make_service(repo: MockSecretRepoTrait) -> SecretStoreImpl {
    SecretStoreImpl::new(Arc::new(repo))
}

fn make_secret_entry(tenant: &str, key: Key, value: SecretValue) -> SecretEntry {
    SecretEntry {
        metadata: metadata(tenant, key),
        value,
    }
}

fn make_new_cmd(key: Key) -> NewSecretCommand {
    NewSecretCommand {
        key,
        value: SecretValue::new(serde_json::json!({"api_token": "super-secret"})),
        description: Some("test secret".to_string()),
    }
}

fn make_edit_cmd() -> EditSecretCommand {
    EditSecretCommand {
        value: SecretValue::new(serde_json::json!({"api_token": "updated-secret"})),
        expected_version: Version::INITIAL,
        description: Some("updated secret description".to_string()),
    }
}

/// Reading a secret of another tenant is not found: the lookup only searches the caller's
/// tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockSecretRepoTrait::new();
    let key = test_key("1");
    repo.expect_get_secret_by_key()
        .withf(move |tenant, k| tenant == "tenant-2" && k == &test_key("1"))
        .returning(|_, _| Ok(None));

    let svc = make_service(repo);
    assert!(
        svc.read(&TestUsers::user("tenant-2", "/admin/tenant-2"), &key)
            .await
            .is_err()
    );
}

/// A non-admin listing another tenant is rejected before touching the repository.
#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockSecretRepoTrait::new();
    let svc = make_service(repo);

    let filter = PrefixFilter {
        prefix: None,
        user_id: Some("tenant-foreign".to_string()),
    };

    let result = svc.list(&TestUsers::user("tenant-1", "/admin/tenant-1"), &filter).await;
    assert!(result.is_err());
}

/// Updating a secret of another tenant is not found and changes nothing.
#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut repo = MockSecretRepoTrait::new();
    let key = test_key("1");
    repo.expect_put_secret()
        .withf(move |tenant, k, _| tenant == "tenant-2" && k == &test_key("1"))
        .returning(|_, _, _| Err(SecretRepoErrors::SecretNotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.update(&TestUsers::user("tenant-2", "/admin/tenant-2"), &key, &make_edit_cmd())
            .await
            .is_err()
    );
}

/// Deleting a secret of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockSecretRepoTrait::new();
    let key = test_key("1");
    repo.expect_delete_secret()
        .withf(move |tenant, k| tenant == "tenant-2" && k == &test_key("1"))
        .returning(|_, _| Err(SecretRepoErrors::SecretNotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.delete(&TestUsers::user("tenant-2", "/admin/tenant-2"), &key)
            .await
            .is_err()
    );
}

/// A batch read only returns secrets of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockSecretRepoTrait::new();
    let key = test_key("1");
    repo.expect_get_batch_secrets()
        .withf(move |tenant, keys| tenant == "tenant-2" && *keys == [test_key("1")])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(repo);
    let entries = svc
        .batch(&TestUsers::user("tenant-2", "/admin/tenant-2"), &[key])
        .await
        .unwrap();
    assert!(entries.is_empty());
}

/// An entry is always created as the caller's.
#[tokio::test]
async fn create_belongs_to_the_caller() {
    let mut repo = MockSecretRepoTrait::new();
    let key = test_key("1");
    repo.expect_create_secret()
        .withf(|user_id, _| user_id == "tenant-2")
        .returning(|tenant, cmd| {
            Ok(make_secret_entry(
                tenant,
                cmd.key.clone(),
                cmd.value.clone(),
            ))
        });

    let svc = make_service(repo);
    let cmd = make_new_cmd(key.clone());
    let entry = svc
        .create(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(entry.metadata.user_id, "tenant-2");
}

/// The root lists without an owner filter.
#[tokio::test]
async fn admin_can_query_cross_tenant() {
    let mut repo = MockSecretRepoTrait::new();
    repo.expect_get_all_secrets()
        .withf(|f| f.user_id.is_none())
        .returning(|_| Ok(vec![]));

    let svc = make_service(repo);
    let result = svc
        .list(&TestUsers::user("admin-tenant", "/admin"), &PrefixFilter::default())
        .await;
    assert!(result.is_ok());
}

/// The root creates its own entries too.
#[tokio::test]
async fn root_creates_its_own_secrets() {
    let mut repo = MockSecretRepoTrait::new();
    let key = test_key("1");
    repo.expect_create_secret()
        .withf(|user_id, _| user_id == "admin-tenant")
        .returning(|tenant, cmd| {
            Ok(make_secret_entry(
                tenant,
                cmd.key.clone(),
                cmd.value.clone(),
            ))
        });

    let svc = make_service(repo);
    let cmd = make_new_cmd(key);
    let entry = svc.create(&TestUsers::user("admin-tenant", "/admin"), &cmd).await.unwrap();
    assert_eq!(entry.metadata.user_id, "admin-tenant");
}
