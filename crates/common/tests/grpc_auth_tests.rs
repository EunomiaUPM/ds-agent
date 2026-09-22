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

use std::sync::Arc;

use common::auth::grpc::GrpcAuth;
use common::auth::{AccessScope, Claims, OauthTokenValidator, RbacRole};
use tonic::metadata::MetadataMap;
use tonic::Code;
use ymir::errors::{Errors, Outcome};

struct MockValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for MockValidator {
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
    GrpcAuth::new(Arc::new(MockValidator))
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

#[tokio::test]
async fn missing_token_is_unauthenticated() {
    let err = auth()
        .scope(&meta(None, Some("tenant-42")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn invalid_token_is_unauthenticated() {
    let err = auth().scope(&meta(Some("nope"), None)).await.unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
    assert_eq!(err.message(), "invalid token");
}

#[tokio::test]
async fn expired_claims_are_unauthenticated() {
    let err = auth()
        .scope(&meta(Some("expired"), None))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn missing_tenant_falls_back_to_claims_tenant() {
    let scope = auth().scope(&meta(Some("owner"), None)).await.unwrap();
    assert_eq!(scope.acting_tenant(), "tenant-42");
    assert_eq!(scope.role(), RbacRole::Owner);
}

#[tokio::test]
async fn own_tenant_is_accepted() {
    let scope = auth()
        .scope(&meta(Some("owner"), Some("tenant-42")))
        .await
        .unwrap();
    assert_eq!(scope.acting_tenant(), "tenant-42");
}

#[tokio::test]
async fn foreign_tenant_is_permission_denied_for_non_admin() {
    let err = auth()
        .scope(&meta(Some("owner"), Some("other-tenant")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn admin_may_act_on_any_tenant() {
    let scope = auth()
        .scope(&meta(Some("admin"), Some("other-tenant")))
        .await
        .unwrap();
    assert_eq!(scope.acting_tenant(), "other-tenant");
    assert!(scope.is_admin());
}

#[tokio::test]
async fn malformed_tenant_is_invalid_argument() {
    let err = auth()
        .scope(&meta(Some("admin"), Some("bad!tenant@id")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
}

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
