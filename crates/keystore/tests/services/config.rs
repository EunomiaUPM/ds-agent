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

//! ConfigStoreImpl with a mocked repository: only an admin may read the application config.

use std::sync::Arc;

use common::test_utils::scopes::TestScopes;
use keystore::data::repo::config::MockKeystoreConfigRepo;
use keystore::services::config::ConfigStore;
use keystore::services::config::config::ConfigStoreImpl;
use ymir::errors::Errors;

/// Owners and readers are rejected before the repository is touched.
#[tokio::test]
async fn non_admins_cannot_read_the_config() {
    let svc = ConfigStoreImpl::new(Arc::new(MockKeystoreConfigRepo::new()));
    assert!(
        svc.get_application_config(&TestScopes::owner("tenant-1"))
            .await
            .is_err()
    );
    assert!(
        svc.get_application_config(&TestScopes::reader("tenant-1"))
            .await
            .is_err()
    );
}

/// An admin reaches the repository, and its result is returned as is.
#[tokio::test]
async fn admin_reads_through_the_repository() {
    let mut repo = MockKeystoreConfigRepo::new();
    repo.expect_get_config()
        .times(1)
        .returning(|| Err(Errors::crazy("vault unreachable", None)));
    let svc = ConfigStoreImpl::new(Arc::new(repo));

    let err = svc
        .get_application_config(&TestScopes::admin())
        .await
        .unwrap_err();
    assert!(format!("{err:?}").contains("vault unreachable"), "{err:?}");
}
