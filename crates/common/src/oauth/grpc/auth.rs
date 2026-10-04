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

//! Bearer-token validation into the caller's `UserInfo` over tonic `MetadataMap`.

use std::sync::Arc;

use tonic::metadata::MetadataMap;
use tonic::Status;

use ymir::types::oauth::UserInfo;

use crate::oauth::TokenValidatorTrait;
use crate::oauth::AUTHORIZATION_HEADER;

/// Validates bearer tokens from metadata into the caller's `UserInfo`.
#[derive(Clone)]
pub struct GrpcAuth {
    validator: Arc<dyn TokenValidatorTrait>,
}

impl GrpcAuth {
    pub fn new(validator: Arc<dyn TokenValidatorTrait>) -> Self {
        Self { validator }
    }

    /// Validates the bearer token carried in metadata and returns the user behind it.
    pub async fn user(&self, meta: &MetadataMap) -> Result<UserInfo, Status> {
        let token = Self::bearer(meta).ok();
        self.validator
            .validate_token(token)
            .await
            .map_err(|e| Status::unauthenticated(e.reason().to_string()))
    }

    /// Extracts the bearer token from the `authorization` metadata entry.
    pub fn bearer(meta: &MetadataMap) -> Result<&str, Status> {
        meta.get(AUTHORIZATION_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Status::unauthenticated("missing or malformed Authorization metadata"))
    }
}
