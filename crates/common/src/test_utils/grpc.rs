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

//! Auth doubles for gRPC adapter tests: a token validator keyed by literal token and requests
//! carrying auth metadata.

use serde_json::Map;
use tonic::Request;
use ymir::errors::{Errors, Outcome};

use crate::oauth::{RolePath, OauthTokenValidatorTrait, UserInfo};

/// User behind the `user` token.
pub const USER_ID: &str = "user-1";
/// Role of [`USER_ID`].
pub const USER_ROLE: &str = "/admin/company/team";
/// User behind the `root` token.
pub const ROOT_ID: &str = "root";

/// Owner of the `user` token's records as the agents' tables keep it (stage A: the user id).
/// Provisional, for the agents' tests written against tenants.
pub const TENANT: &str = USER_ID;
/// An owner none of the stub tokens is. Provisional, as [`TENANT`].
pub const OTHER_TENANT: &str = "tenant-2";

/// Accepts `user` as [`USER_ID`] under [`USER_ROLE`] and `root` as a superuser; rejects
/// anything else.
pub struct StubTokenValidator;

#[async_trait::async_trait]
impl OauthTokenValidatorTrait for StubTokenValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        let (user_id, role) = match token.unwrap_or_default() {
            "user" => (USER_ID, USER_ROLE.parse::<RolePath>()?),
            "root" => (ROOT_ID, RolePath::root()),
            _ => return Err(Errors::unauthorized("invalid token", None)),
        };
        Ok(UserInfo::new(user_id, None, role, Map::new()))
    }
}

/// Builds gRPC requests with the `authorization` metadata set.
pub struct GrpcRequests;

impl GrpcRequests {
    pub fn with_auth<T>(body: T, token: Option<&str>) -> Request<T> {
        let mut req = Request::new(body);
        if let Some(t) = token {
            req.metadata_mut()
                .insert("authorization", format!("Bearer {t}").parse().unwrap());
        }
        req
    }

    /// Request as [`USER_ID`].
    pub fn user<T>(body: T) -> Request<T> {
        Self::with_auth(body, Some("user"))
    }

    /// Request as the owner of [`TENANT`], i.e. [`USER_ID`]. Provisional, as [`TENANT`].
    pub fn owner<T>(body: T) -> Request<T> {
        Self::user(body)
    }
}
