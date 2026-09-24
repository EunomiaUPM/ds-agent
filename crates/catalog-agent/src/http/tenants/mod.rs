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

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use common::auth::AccessScope;

use crate::http::common::to_camel_case::ToCamelCase;
use crate::services::tenant_provisioning::TenantProvisioningServiceTrait;

#[derive(Clone)]
pub(crate) struct TenantRouter {
    service: Arc<dyn TenantProvisioningServiceTrait>,
}

impl TenantRouter {
    pub(crate) fn new(service: Arc<dyn TenantProvisioningServiceTrait>) -> Self {
        Self { service }
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/{tenant_id}/provision", post(Self::handle_provision))
            .with_state(self)
    }

    async fn handle_provision(
        State(state): State<Self>,
        scope: AccessScope,
        Path(tenant_id): Path<String>,
    ) -> impl IntoResponse {
        match state.service.provision(&scope, &tenant_id).await {
            Ok(provisioned) => (StatusCode::OK, Json(ToCamelCase(provisioned))).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
