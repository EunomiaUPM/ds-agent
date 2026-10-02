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

use tonic::Request;
use ymir::errors::{Errors, Outcome};

use crate::auth::{Claims, OauthTokenValidator, RbacRole};

/// Tenant of every token the stub validator accepts.
pub const TENANT: &str = "tenant-1";
/// A tenant the stub tokens do not belong to.
pub const OTHER_TENANT: &str = "tenant-2";

/// Accepts `owner` as the owner of [`TENANT`] and `admin` as an admin; rejects anything else.
pub struct StubTokenValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for StubTokenValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        let role = match token {
            "owner" => RbacRole::Owner,
            "admin" => RbacRole::Admin,
            _ => return Err(Errors::unauthorized("invalid token", None)),
        };
        Ok(Claims {
            sub: TENANT.to_string(),
            role,
            iat: 1000,
            exp: 9_999_999_999,
        })
    }
}

/// Builds gRPC requests with the `authorization` and `x-tenant-id` metadata set.
pub struct GrpcRequests;

impl GrpcRequests {
    pub fn with_auth<T>(body: T, token: Option<&str>, tenant: Option<&str>) -> Request<T> {
        let mut req = Request::new(body);
        if let Some(t) = token {
            req.metadata_mut()
                .insert("authorization", format!("Bearer {t}").parse().unwrap());
        }
        if let Some(t) = tenant {
            req.metadata_mut().insert("x-tenant-id", t.parse().unwrap());
        }
        req
    }

    /// Request as the owner of [`TENANT`] acting on its own tenant.
    pub fn owner<T>(body: T) -> Request<T> {
        Self::with_auth(body, Some("owner"), Some(TENANT))
    }
}
