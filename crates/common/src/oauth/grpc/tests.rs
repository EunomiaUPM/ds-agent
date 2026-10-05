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

//! GrpcAuth: bearer extraction and the UserInfo read from metadata, with a stub validator.

use std::sync::Arc;

use serde_json::Map;
use tonic::metadata::MetadataMap;
use tonic::Code;
use ymir::errors::{Errors, Outcome};

use crate::oauth::grpc::GrpcAuth;
use crate::oauth::{RolePath, OauthTokenValidatorTrait, UserInfo};

/// Accepts a root and a regular user token; anything else is unauthorized.
struct StubValidator;

#[async_trait::async_trait]
impl OauthTokenValidatorTrait for StubValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        match token.unwrap_or_default() {
            "root" => Ok(UserInfo::new("root", None, RolePath::root(), Map::new())),
            "user" => Ok(UserInfo::new("ana", None, "/admin/upm/dit".parse()?, Map::new())),
            _ => Err(Errors::unauthorized("invalid token", None)),
        }
    }
}

fn auth() -> GrpcAuth {
    GrpcAuth::new(Arc::new(StubValidator))
}

fn meta(token: Option<&str>) -> MetadataMap {
    let mut m = MetadataMap::new();
    if let Some(t) = token {
        m.insert("authorization", format!("Bearer {t}").parse().unwrap());
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
    let err = auth().user(&meta(None)).await.unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A token the validator rejects is Unauthenticated and keeps its message.
#[tokio::test]
async fn invalid_token_is_unauthenticated() {
    let err = auth().user(&meta(Some("nope"))).await.unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
    assert_eq!(err.message(), "invalid token");
}

/// A valid token gives its user and role.
#[tokio::test]
async fn valid_token_gives_its_user() {
    let user = auth().user(&meta(Some("user"))).await.unwrap();
    assert_eq!(user.user_id(), "ana");
    assert_eq!(user.role().as_str(), "/admin/upm/dit");
    assert!(!user.is_root());

    let root = auth().user(&meta(Some("root"))).await.unwrap();
    assert!(root.is_root());
}
