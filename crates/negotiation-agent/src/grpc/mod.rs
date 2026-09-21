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

use axum::http::StatusCode;
use common::auth::OauthTokenValidator;
use common::auth::access::AccessScope;
use std::sync::Arc;
use tonic::Status;
use ymir::errors::Errors;

pub mod api {
    pub mod negotiation_agent {
        tonic::include_proto!("negotiation_agent.v1");
    }

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("negotiation_descriptor");
}

pub(crate) mod agreement;
pub(crate) mod mappers;
pub(crate) mod negotiation_message;
pub(crate) mod negotiation_process;
pub(crate) mod offer;

/// Trait to convert domain `Errors` into gRPC `Status`.
pub(crate) trait IntoGrpcStatus {
    fn into_status(self) -> Status;
}

impl IntoGrpcStatus for Errors {
    fn into_status(self) -> Status {
        let message = self.reason().to_string();
        match self.info().status_code {
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

/// Helper for authenticating and constructing `AccessScope` from gRPC metadata.
pub(crate) struct GrpcAuthHelper;

impl GrpcAuthHelper {
    /// Extracts Bearer token and x-tenant-id from gRPC metadata and builds `AccessScope`.
    pub(crate) async fn extract_scope(
        validator: &Arc<dyn OauthTokenValidator>,
        meta: &tonic::metadata::MetadataMap,
    ) -> Result<AccessScope, Status> {
        let token = meta
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("missing Authorization metadata"))?;

        let claims = validator
            .validate_token(token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        common::auth::validators::AuthValidators::claims_validator()
            .validate(&claims)
            .map_err(|vs| Status::unauthenticated(vs.to_string()))?;

        let tenant_raw = meta
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::invalid_argument("missing x-tenant-id metadata"))?;

        let tenant_id = tenant_raw.to_string();
        common::auth::validators::AuthValidators::tenant_id_validator()
            .validate(&tenant_id)
            .map_err(|vs| Status::invalid_argument(vs.to_string()))?;

        if !claims.is_admin() && claims.tenant_id() != tenant_id {
            return Err(Status::permission_denied(
                "forbidden: caller tenant does not match requested tenant",
            ));
        }

        Ok(AccessScope::new(&claims, &tenant_id))
    }
}
