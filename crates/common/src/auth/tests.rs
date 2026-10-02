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

//! AccessScope, Rbac, Claims, AuthValidators and AuthRules on plain claims, without transport.

use crate::auth::validators::AuthValidators;
use crate::auth::{AccessScope, AuthRules, Claims, Rbac, RbacRole};
use crate::validation::codes;

fn claims(sub: &str, role: RbacRole) -> Claims {
    Claims {
        sub: sub.to_string(),
        role,
        iat: 1000,
        exp: 9999999999,
    }
}

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

/// An in-process service scope equals what the HTTP extractor derives from a service token
/// pinned to a tenant.
#[test]
fn service_scope_matches_service_token_with_tenant_header() {
    let remote =
        AccessScope::from_tenant_header(&service_token_claims("admin"), Some("tenant-a")).unwrap();
    let local = AccessScope::service("tenant-a");
    assert_same(&local, &remote);
    assert_eq!(local.tenant_filter(), Some("tenant-a"));
}

/// A cross-tenant service scope equals a service token sent without tenant header.
#[test]
fn cross_tenant_scope_matches_service_token_without_tenant_header() {
    let remote = AccessScope::from_tenant_header(&service_token_claims("admin"), None).unwrap();
    let local = AccessScope::service_cross_tenant("admin");
    assert_same(&local, &remote);
    assert_eq!(local.tenant_filter(), None);
}

/// An Admin scope built from its role sees every tenant, unlike a service scope.
#[test]
fn from_role_admin_is_not_a_tenant_scoped_service_call() {
    let unpinned = AccessScope::from_role(RbacRole::Admin, "tenant-a");
    assert_eq!(unpinned.tenant_filter(), None);
    assert!(!AccessScope::service("tenant-a").permits("tenant-b"));
    assert!(unpinned.permits("tenant-b"));
}

/// The tenant header falls back to the token's tenant, and only an Admin may name another one.
#[test]
fn from_tenant_header_is_shared_core_rule() {
    let owner = claims("tenant-42", RbacRole::Owner);
    assert_eq!(
        AccessScope::from_tenant_header(&owner, None)
            .unwrap()
            .acting_tenant(),
        "tenant-42"
    );
    assert!(AccessScope::from_tenant_header(&owner, Some("tenant-42")).is_ok());
    assert!(AccessScope::from_tenant_header(&owner, Some("other")).is_err());

    let admin = claims("admin-tenant", RbacRole::Admin);
    assert_eq!(
        AccessScope::from_tenant_header(&admin, Some("other"))
            .unwrap()
            .acting_tenant(),
        "other"
    );
    assert!(AccessScope::from_tenant_header(&admin, Some("bad!tenant@id")).is_err());
}

/// Rbac and the read/write scopes confine non-admins to their tenant and readers to reading.
#[test]
fn rbac_and_scope_confine_non_admins_to_their_tenant() {
    let admin_claims = Claims {
        sub: "admin-system".to_string(),
        role: RbacRole::Admin,
        iat: 100,
        exp: 200,
    };
    let user_claims = Claims {
        sub: "tenant-alpha".to_string(),
        role: RbacRole::Owner,
        iat: 100,
        exp: 200,
    };
    let reader_claims = Claims {
        sub: "tenant-alpha".to_string(),
        role: RbacRole::Reader,
        iat: 100,
        exp: 200,
    };

    assert!(Rbac::require_admin(&admin_claims).is_ok());
    assert!(Rbac::require_admin(&user_claims).is_err());

    assert!(Rbac::require_read(&admin_claims, "any-tenant").is_ok());
    assert!(Rbac::require_read(&user_claims, "tenant-alpha").is_ok());
    assert!(Rbac::require_read(&user_claims, "tenant-beta").is_err());

    assert!(Rbac::require_write(&admin_claims, "any-tenant").is_ok());
    assert!(Rbac::require_write(&user_claims, "tenant-alpha").is_ok());
    assert!(Rbac::require_write(&user_claims, "tenant-beta").is_err());
    assert!(Rbac::require_write(&reader_claims, "tenant-alpha").is_err());

    let admin_scope = AccessScope::for_read(&admin_claims, "foreign-tenant").unwrap();
    assert_eq!(admin_scope.tenant_filter(), None);
    assert!(admin_scope.permits("any-other-tenant"));

    let user_scope = AccessScope::for_write(&user_claims, "tenant-alpha").unwrap();
    assert_eq!(user_scope.tenant_filter(), Some("tenant-alpha"));
    assert!(user_scope.permits("tenant-alpha"));
    assert!(!user_scope.permits("tenant-beta"));
}

/// Claims expose their tenant, admin flag and expiry; roles their name and write right.
#[test]
fn claims_and_roles_expose_tenant_admin_and_expiry() {
    let role = RbacRole::Owner;
    assert_eq!(role.as_str(), "owner");
    assert!(!role.is_admin());
    assert!(role.can_write());

    let claims = Claims {
        sub: "tenant-123".to_string(),
        role: RbacRole::Admin,
        iat: 1000,
        exp: 2000,
    };
    assert_eq!(claims.tenant_id(), "tenant-123");
    assert!(claims.is_admin());
    assert!(!claims.is_expired(1500));
    assert!(claims.is_expired(2500));
}

/// The claims validator rejects a blank subject and an expired token; the tenant one a bad id.
#[test]
fn claims_and_tenant_validators_reject_bad_input() {
    let now = 1000;
    let claims_val = AuthValidators::claims_validator_at(now);

    let valid_claims = Claims {
        sub: "tenant-42".to_string(),
        role: RbacRole::Owner,
        iat: 500,
        exp: 1500,
    };
    assert!(claims_val.validate(&valid_claims).is_ok());

    let empty_sub = Claims {
        sub: "  ".to_string(),
        role: RbacRole::Owner,
        iat: 500,
        exp: 1500,
    };
    let vs = claims_val.validate(&empty_sub).unwrap_err();
    assert_eq!(vs.code(), Some(codes::MISSING));

    let expired = Claims {
        sub: "tenant-42".to_string(),
        role: RbacRole::Owner,
        iat: 100,
        exp: 900,
    };
    let vs = claims_val.validate(&expired).unwrap_err();
    assert_eq!(vs.code(), Some(codes::NOT_ALLOWED));

    let tenant_val = AuthValidators::tenant_id_validator();
    assert!(tenant_val.validate(&"valid-tenant.1".to_string()).is_ok());
    assert!(tenant_val.validate(&"".to_string()).is_err());
    assert!(tenant_val.validate(&"invalid/tenant#".to_string()).is_err());
}

/// Service-layer checks: readers cannot write, non-admins stay in their tenant, and the
/// create and query tenants resolve per role.
#[test]
fn service_scope_checks_role_and_tenant() {
    let admin_scope = AccessScope::from_role(RbacRole::Admin, "system");
    let owner_scope = AccessScope::from_role(RbacRole::Owner, "tenant-a");
    let reader_scope = AccessScope::from_role(RbacRole::Reader, "tenant-a");

    assert!(admin_scope.require_read().is_ok());
    assert!(owner_scope.require_read().is_ok());
    assert!(reader_scope.require_read().is_ok());

    assert!(admin_scope.require_write().is_ok());
    assert!(owner_scope.require_write().is_ok());
    assert!(reader_scope.require_write().is_err());

    assert!(admin_scope.ensure_tenant_access("tenant-b").is_ok());
    assert!(owner_scope.ensure_tenant_access("tenant-a").is_ok());
    assert!(owner_scope.ensure_tenant_access("tenant-b").is_err());

    assert!(admin_scope.require_read_tenant("tenant-b").is_ok());
    assert!(owner_scope.require_read_tenant("tenant-a").is_ok());
    assert!(owner_scope.require_read_tenant("tenant-b").is_err());
    assert!(admin_scope.require_write_tenant("tenant-b").is_ok());
    assert!(owner_scope.require_write_tenant("tenant-a").is_ok());
    assert!(owner_scope.require_write_tenant("tenant-b").is_err());
    assert!(reader_scope.require_write_tenant("tenant-a").is_err());

    assert_eq!(
        admin_scope.resolve_create_tenant(Some("tenant-b")).unwrap(),
        "tenant-b"
    );
    assert_eq!(admin_scope.resolve_create_tenant(None).unwrap(), "system");
    assert_eq!(
        owner_scope.resolve_create_tenant(Some("tenant-b")).unwrap(),
        "tenant-a"
    );
    assert_eq!(owner_scope.resolve_create_tenant(None).unwrap(), "tenant-a");
    assert!(reader_scope.resolve_create_tenant(None).is_err());
    assert!(reader_scope
        .resolve_create_tenant(Some("tenant-a"))
        .is_err());

    assert_eq!(
        admin_scope.resolve_query_tenant(Some("tenant-b")).unwrap(),
        Some("tenant-b".to_string())
    );
    assert_eq!(admin_scope.resolve_query_tenant(None).unwrap(), None);
    assert_eq!(
        owner_scope.resolve_query_tenant(None).unwrap(),
        Some("tenant-a".to_string())
    );
    assert_eq!(
        owner_scope.resolve_query_tenant(Some("tenant-a")).unwrap(),
        Some("tenant-a".to_string())
    );
    assert!(owner_scope.resolve_query_tenant(Some("tenant-b")).is_err());
    assert_eq!(
        reader_scope.resolve_query_tenant(None).unwrap(),
        Some("tenant-a".to_string())
    );
    assert!(reader_scope.resolve_query_tenant(Some("tenant-b")).is_err());
}

/// AuthRules check token expiry, audience, issuer and role membership.
#[test]
fn auth_rules_check_expiry_audience_issuer_and_role() {
    let now = 1000;
    assert!(AuthRules::token_not_expired(1500, now, "exp").is_ok());
    let expired = AuthRules::token_not_expired(900, now, "exp").unwrap_err();
    assert_eq!(expired.code(), Some(codes::NOT_ALLOWED));

    assert!(
        AuthRules::audience_matches("https://agent.local", "https://agent.local", "aud").is_ok()
    );
    assert!(AuthRules::audience_matches("https://other", "https://agent.local", "aud").is_err());

    assert!(AuthRules::issuer_matches("https://idp.local", "https://idp.local", "iss").is_ok());
    assert!(AuthRules::issuer_matches("https://fake", "https://idp.local", "iss").is_err());

    let roles = vec!["admin".to_string(), "reader".to_string()];
    assert!(AuthRules::has_role(&roles, "admin", "roles").is_ok());
    assert!(AuthRules::has_role(&roles, "writer", "roles").is_err());
}

/// A foreign record looks like a missing one, so probing reveals nothing.
#[test]
fn ensure_visible_hides_foreign_records_as_not_found() {
    let owner = AccessScope::from_role(RbacRole::Owner, "tenant-a");
    assert!(owner.ensure_visible("tenant-a", "urn:x:1").is_ok());
    match owner.ensure_visible("tenant-b", "urn:x:1").unwrap_err() {
        ymir::errors::Errors::MissingResourceError { info, .. } => {
            assert_eq!(info.status_code, 404)
        }
        other => panic!("expected not found, got {other:?}"),
    }
    let admin = AccessScope::from_role(RbacRole::Admin, "system");
    assert!(admin.ensure_visible("tenant-b", "urn:x:1").is_ok());
}

/// An admin that names a tenant in the header is pinned to it and sees nothing else.
#[test]
fn admin_naming_a_tenant_is_pinned_to_it() {
    let admin = claims("admin-tenant", RbacRole::Admin);
    let pinned = AccessScope::from_tenant_header(&admin, Some("tenant-a")).unwrap();
    assert_eq!(pinned.tenant_filter(), Some("tenant-a"));
    assert!(!pinned.permits("tenant-b"));
    assert!(pinned.ensure_visible("tenant-b", "urn:x:1").is_err());
    assert!(
        pinned.is_admin(),
        "still an admin for admin-only operations"
    );
}

/// Only an admin scope passes `require_admin`; the rejection is a 403.
#[test]
fn require_admin_rejects_non_admins_with_forbidden() {
    assert!(AccessScope::from_role(RbacRole::Admin, "system")
        .require_admin()
        .is_ok());
    let err = AccessScope::from_role(RbacRole::Owner, "tenant-a")
        .require_admin()
        .unwrap_err();
    match err {
        ymir::errors::Errors::ForbiddenError { info, .. } => assert_eq!(info.status_code, 403),
        other => panic!("expected forbidden, got {other:?}"),
    }
}
