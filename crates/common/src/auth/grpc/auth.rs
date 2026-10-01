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

//! Bearer-token validation and tenant scoping over tonic `MetadataMap`.

use std::sync::Arc;

use tonic::metadata::MetadataMap;
use tonic::Status;
use ymir::errors::Errors;

use crate::auth::access::AccessScope;
use crate::auth::claims::Claims;
use crate::auth::token::OauthTokenValidator;
use crate::auth::validators::AuthValidators;
use crate::auth::{AUTHORIZATION_HEADER, TENANT_HEADER};
use crate::grpc::IntoStatus;

/// Validates bearer tokens and the `x-tenant-id` metadata into an `AccessScope`.
#[derive(Clone)]
pub struct GrpcAuth {
    validator: Arc<dyn OauthTokenValidator>,
}

impl GrpcAuth {
    pub fn new(validator: Arc<dyn OauthTokenValidator>) -> Self {
        Self { validator }
    }

    /// Builds the caller's access scope from validated metadata.
    pub async fn scope(&self, meta: &MetadataMap) -> Result<AccessScope, Status> {
        let claims = self.claims(meta).await?;
        AccessScope::from_tenant_header(&claims, Self::tenant(meta)).map_err(Errors::into_status)
    }

    /// Validates the bearer token carried in metadata and returns its claims.
    pub async fn claims(&self, meta: &MetadataMap) -> Result<Claims, Status> {
        let token = Self::bearer(meta)?;
        let claims = self
            .validator
            .validate_token(token)
            .await
            .map_err(|e| Status::unauthenticated(e.reason().to_string()))?;
        AuthValidators::claims_validator()
            .validate(&claims)
            .map_err(|vs| Status::unauthenticated(vs.to_string()))?;
        Ok(claims)
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

    /// Requested tenant from the `x-tenant-id` metadata entry, if present.
    pub fn tenant(meta: &MetadataMap) -> Option<&str> {
        meta.get(TENANT_HEADER).and_then(|v| v.to_str().ok())
    }
}
