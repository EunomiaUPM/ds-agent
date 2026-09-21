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

//! Caller authentication for gRPC handlers: builds the tenant `AccessScope` from request metadata.

use axum::http::StatusCode;
use common::auth::claims::Claims;
use common::auth::validators::AuthValidators;
use common::auth::{AccessScope, OauthTokenValidator};
use std::sync::Arc;
use tonic::metadata::MetadataMap;
use tonic::Status;
use ymir::errors::Errors;

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
        let (claims, tenant) = self.extract_auth(meta).await?;
        Ok(AccessScope::new(&claims, &tenant))
    }

    async fn extract_auth(&self, meta: &MetadataMap) -> Result<(Claims, String), Status> {
        let token = meta
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("missing Authorization metadata"))?;

        let claims = self
            .validator
            .validate_token(token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        AuthValidators::claims_validator()
            .validate(&claims)
            .map_err(|vs| Status::unauthenticated(vs.to_string()))?;

        let tenant_id = meta
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::invalid_argument("missing x-tenant-id metadata"))?
            .to_string();
        AuthValidators::tenant_id_validator()
            .validate(&tenant_id)
            .map_err(|vs| Status::invalid_argument(vs.to_string()))?;

        if !claims.is_admin() && claims.tenant_id() != tenant_id {
            return Err(Status::permission_denied(
                "forbidden: caller tenant does not match requested tenant",
            ));
        }

        Ok((claims, tenant_id))
    }
}

/// Maps domain errors onto gRPC status codes.
pub struct StatusMapper;

impl StatusMapper {
    pub fn to_status(err: Errors) -> Status {
        let message = err.reason().to_string();
        match err.info().status_code {
            StatusCode::NOT_FOUND => Status::not_found(message),
            StatusCode::FORBIDDEN => Status::permission_denied(message),
            StatusCode::UNAUTHORIZED => Status::unauthenticated(message),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                Status::invalid_argument(message)
            }
            StatusCode::PRECONDITION_FAILED => Status::failed_precondition(message),
            _ => Status::internal(message),
        }
    }
}
