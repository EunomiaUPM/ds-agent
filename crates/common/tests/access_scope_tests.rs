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

//! In-process facade scopes must equal what the HTTP extractor derives from the service token.

use common::auth::{AccessScope, Claims, RbacRole};

fn service_token_claims(home_tenant: &str) -> Claims {
    Claims {
        sub: home_tenant.to_string(),
        role: RbacRole::Admin,
        iat: 0,
        exp: i64::MAX,
    }
}

fn assert_same(local: &AccessScope, remote: &AccessScope) {
    assert_eq!(local.role(), remote.role());
    assert_eq!(local.acting_tenant(), remote.acting_tenant());
    assert_eq!(local.tenant_filter(), remote.tenant_filter());
}

#[test]
fn service_scope_matches_service_token_with_tenant_header() {
    let remote =
        AccessScope::from_tenant_header(&service_token_claims("admin"), Some("tenant-a")).unwrap();
    let local = AccessScope::service("tenant-a");
    assert_same(&local, &remote);
    assert_eq!(local.tenant_filter(), Some("tenant-a"));
}

#[test]
fn cross_tenant_scope_matches_service_token_without_tenant_header() {
    let remote = AccessScope::from_tenant_header(&service_token_claims("admin"), None).unwrap();
    let local = AccessScope::service_cross_tenant("admin");
    assert_same(&local, &remote);
    assert_eq!(local.tenant_filter(), None);
}

#[test]
fn from_role_admin_is_not_a_tenant_scoped_service_call() {
    // An unpinned Admin sees every tenant: the pitfall `AccessScope::service` avoids.
    let unpinned = AccessScope::from_role(RbacRole::Admin, "tenant-a");
    assert_eq!(unpinned.tenant_filter(), None);
    assert!(!AccessScope::service("tenant-a").permits("tenant-b"));
    assert!(unpinned.permits("tenant-b"));
}
