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

use common::oauth::{OauthTokenValidatorTrait, RolePath, UserInfo};
use serde_json::Map;
use ymir::errors::{Errors, Outcome};

/// Accepts `valid-jwt-token` and any `pat_` token as the root `user-admin-123`.
pub struct StubTokenValidator;

#[async_trait::async_trait]
impl OauthTokenValidatorTrait for StubTokenValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        let token = token.unwrap_or_default();
        if token == "valid-jwt-token" || token.starts_with("pat_") {
            Ok(UserInfo::new("user-admin-123", None, RolePath::root(), Map::new()))
        } else {
            Err(Errors::unauthorized("invalid token", None))
        }
    }
}
