/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::instantiation_engine::{NewPolicyInstantiationDto, PolicyInstantiationTrait};
use crate::entities::policy_templates::{NewPolicyTemplateDto, PolicyTemplateEntityTrait};
use crate::http::common::to_camel_case::ToCamelCase;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequestsAsString;
use common::config::services::CatalogConfig;
use common::errors::CommonErrors;
use serde::Deserialize;
use std::sync::Arc;
use ymir::errors::Errors;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct PolicyTemplateEntityRouter {
    service: Arc<dyn PolicyTemplateEntityTrait>,
    policy_engine: Arc<dyn PolicyInstantiationTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

#[derive(Deserialize)]
pub struct SilentParams {
    pub silent: Option<bool>,
}

impl FromRef<PolicyTemplateEntityRouter> for Arc<dyn PolicyTemplateEntityTrait> {
    fn from_ref(state: &PolicyTemplateEntityRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<PolicyTemplateEntityRouter> for Arc<dyn PolicyInstantiationTrait> {
    fn from_ref(state: &PolicyTemplateEntityRouter) -> Self {
        state.policy_engine.clone()
    }
}

impl FromRef<PolicyTemplateEntityRouter> for Arc<CatalogConfig> {
    fn from_ref(state: &PolicyTemplateEntityRouter) -> Self {
        state.config.clone()
    }
}

impl PolicyTemplateEntityRouter {
    pub fn new(
        service: Arc<dyn PolicyTemplateEntityTrait>,
        policy_engine: Arc<dyn PolicyInstantiationTrait>,
        config: Arc<CatalogConfig>,
    ) -> Self {
        Self {
            service,
            policy_engine,
            config,
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_policy_templates))
            .route("/", post(Self::handle_create_policy_template))
            .route("/batch", post(Self::handle_get_batch_policy_templates))
            .route("/{id}", get(Self::handle_get_policy_template_by_id))
            .route(
                "/{id}/{version}",
                get(Self::handle_get_policy_template_by_id_and_version),
            )
            .route(
                "/{id}/{version}",
                delete(Self::handle_delete_policy_template_by_id_and_version),
            )
            .route(
                "/instantiate-odrl-offer",
                post(Self::handle_instantiate_offer),
            )
            .with_state(self)
    }

    async fn handle_get_all_policy_templates(
        State(state): State<PolicyTemplateEntityRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_all_policy_templates(params.limit, params.page)
            .await
        {
            Ok(templates) => (StatusCode::OK, Json(ToCamelCase(templates))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_batch_policy_templates(
        State(state): State<PolicyTemplateEntityRouter>,
        input: Result<Json<BatchRequestsAsString>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.get_batch_policy_templates(&input.ids).await {
            Ok(templates) => (StatusCode::OK, Json(ToCamelCase(templates))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_policy_template_by_id(
        State(state): State<PolicyTemplateEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match state.service.get_policies_template_by_id(&id).await {
            Ok(templates) => (StatusCode::OK, Json(ToCamelCase(templates))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_policy_template_by_id_and_version(
        State(state): State<PolicyTemplateEntityRouter>,
        Path((id, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_policies_template_by_version_and_id(&id, &version)
            .await
        {
            Ok(Some(template)) => (StatusCode::OK, Json(ToCamelCase(template))).into_response(),
            Ok(None) => {
                let err = Errors::missing_resource(id.as_str(), "Policy template not found", None);
                err.into_response()
            }
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_create_policy_template(
        State(state): State<PolicyTemplateEntityRouter>,
        Query(params): Query<SilentParams>,
        input: Result<Json<NewPolicyTemplateDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let silent = params.silent.unwrap_or(false);
        let input = match input {
            Ok(Json(v)) => v,
            Err(e) => {
                if silent {
                    tracing::warn!("Silent mode: Invalid JSON payload ignored: {}", e);
                    // RETORNO TEMPRANO: Devolvemos 200 OK y terminamos la ejecución
                    return (StatusCode::OK).into_response();
                }
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)).into_response();
            }
        };
        match state.service.create_policy_template(&input).await {
            Ok(template) => (StatusCode::OK, Json(ToCamelCase(template))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_delete_policy_template_by_id_and_version(
        State(state): State<PolicyTemplateEntityRouter>,
        Path((id, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state
            .service
            .delete_policy_template_by_version_and_id(&id, &version)
            .await
        {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => return e.into_response(),
        }
    }

    async fn handle_instantiate_offer(
        State(state): State<PolicyTemplateEntityRouter>,
        input: Result<Json<NewPolicyInstantiationDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.policy_engine.instantiate_policy(&input).await {
            Ok(dto) => (StatusCode::ACCEPTED, Json(ToCamelCase(dto))).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
