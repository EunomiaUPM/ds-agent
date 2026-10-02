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

//! Hand-written stubs.

use common::auth::claims::{Claims, RbacRole};
use common::auth::OauthTokenValidator;
use ymir::errors::{Errors, Outcome};

/// Accepts `valid-jwt-token` and any `pat_` token as the admin `user-admin-123`.
pub struct StubTokenValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for StubTokenValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        if token == "valid-jwt-token" || token.starts_with("pat_") {
            Ok(Claims {
                sub: "user-admin-123".to_string(),
                role: RbacRole::Admin,
                iat: 1000,
                exp: 9999999999,
            })
        } else {
            Err(Errors::unauthorized("invalid token", None))
        }
    }
}
