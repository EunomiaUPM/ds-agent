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

//! GrpcAuth: bearer extraction and the AccessScope built from metadata, with a stub validator.

use std::sync::Arc;

use tonic::metadata::MetadataMap;
use tonic::Code;
use ymir::errors::{Errors, Outcome};

use crate::auth::grpc::GrpcAuth;
use crate::auth::{Claims, OauthTokenValidator, RbacRole};

/// Accepts an admin, an owner and an expired owner token; anything else is unauthorized.
struct StubValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for StubValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        match token {
            "admin" => Ok(claims("admin-tenant", RbacRole::Admin)),
            "owner" => Ok(claims("tenant-42", RbacRole::Owner)),
            "expired" => Ok(Claims {
                exp: 1,
                ..claims("tenant-42", RbacRole::Owner)
            }),
            _ => Err(Errors::unauthorized("invalid token", None)),
        }
    }
}

fn claims(sub: &str, role: RbacRole) -> Claims {
    Claims {
        sub: sub.to_string(),
        role,
        iat: 1000,
        exp: 9999999999,
    }
}

fn auth() -> GrpcAuth {
    GrpcAuth::new(Arc::new(StubValidator))
}

fn meta(token: Option<&str>, tenant: Option<&str>) -> MetadataMap {
    let mut m = MetadataMap::new();
    if let Some(t) = token {
        m.insert("authorization", format!("Bearer {t}").parse().unwrap());
    }
    if let Some(t) = tenant {
        m.insert("x-tenant-id", t.parse().unwrap());
    }
    m
}

/// The bearer is trimmed; a non-bearer, blank or missing one is Unauthenticated.
#[test]
fn bearer_extraction_trims_and_rejects_malformed() {
    let mut m = MetadataMap::new();
    m.insert("authorization", "Bearer   spaced   ".parse().unwrap());
    assert_eq!(GrpcAuth::bearer(&m).unwrap(), "spaced");

    let mut basic = MetadataMap::new();
    basic.insert("authorization", "Basic abc".parse().unwrap());
    assert_eq!(
        GrpcAuth::bearer(&basic).unwrap_err().code(),
        Code::Unauthenticated
    );

    let mut empty = MetadataMap::new();
    empty.insert("authorization", "Bearer ".parse().unwrap());
    assert_eq!(
        GrpcAuth::bearer(&empty).unwrap_err().code(),
        Code::Unauthenticated
    );

    assert_eq!(
        GrpcAuth::bearer(&MetadataMap::new()).unwrap_err().code(),
        Code::Unauthenticated
    );
}

/// A call without token is Unauthenticated.
#[tokio::test]
async fn missing_token_is_unauthenticated() {
    let err = auth()
        .scope(&meta(None, Some("tenant-42")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A token the validator rejects is Unauthenticated and keeps its message.
#[tokio::test]
async fn invalid_token_is_unauthenticated() {
    let err = auth().scope(&meta(Some("nope"), None)).await.unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
    assert_eq!(err.message(), "invalid token");
}

/// Valid but expired claims are Unauthenticated.
#[tokio::test]
async fn expired_claims_are_unauthenticated() {
    let err = auth()
        .scope(&meta(Some("expired"), None))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// Without tenant metadata the caller acts on the tenant of its claims.
#[tokio::test]
async fn missing_tenant_falls_back_to_claims_tenant() {
    let scope = auth().scope(&meta(Some("owner"), None)).await.unwrap();
    assert_eq!(scope.acting_tenant(), "tenant-42");
    assert_eq!(scope.role(), RbacRole::Owner);
}

/// A non-admin may name its own tenant.
#[tokio::test]
async fn own_tenant_is_accepted() {
    let scope = auth()
        .scope(&meta(Some("owner"), Some("tenant-42")))
        .await
        .unwrap();
    assert_eq!(scope.acting_tenant(), "tenant-42");
}

/// A non-admin naming another tenant is PermissionDenied.
#[tokio::test]
async fn foreign_tenant_is_permission_denied_for_non_admin() {
    let err = auth()
        .scope(&meta(Some("owner"), Some("other-tenant")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

/// An Admin may act on any tenant.
#[tokio::test]
async fn admin_may_act_on_any_tenant() {
    let scope = auth()
        .scope(&meta(Some("admin"), Some("other-tenant")))
        .await
        .unwrap();
    assert_eq!(scope.acting_tenant(), "other-tenant");
    assert!(scope.is_admin());
}

/// A malformed tenant id is InvalidArgument, even for an Admin.
#[tokio::test]
async fn malformed_tenant_is_invalid_argument() {
    let err = auth()
        .scope(&meta(Some("admin"), Some("bad!tenant@id")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}
