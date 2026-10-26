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

//! The policy template API: templates and their versions, and offers instantiated from them.

use crate::entities::filters::PolicyTemplateFilter;
use crate::entities::policy_instantiation::NewPolicyInstantiationDto;
use crate::entities::policy_templates::NewPolicyTemplateDto;
use crate::http::common::to_camel_case::ToCamelCase;
use crate::services::odrl_policies::views::OdrlPolicyView;
use crate::setup::context::AppContext;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequestsAsString;
use common::query::QuerySpec;
use common::routes::catalog as routes;
use serde::Deserialize;
use std::sync::Arc;
use ymir::errors::Errors;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct PolicyTemplateEntityRouter {
    ctx: Arc<AppContext>,
}

pub type PolicyTemplateQuery = QuerySpec<PolicyTemplateFilter>;

#[derive(Deserialize)]
pub struct SilentParams {
    pub silent: Option<bool>,
}

impl PolicyTemplateEntityRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::entity::ROOT, get(Self::handle_get_all_policy_templates))
            .route(routes::entity::ROOT, post(Self::handle_create_policy_template))
            .route(routes::entity::BATCH, post(Self::handle_get_batch_policy_templates))
            .route(routes::entity::BY_ID, get(Self::handle_get_policy_template_by_id))
            .route(
                routes::policy_templates::BY_ID_AND_VERSION,
                get(Self::handle_get_policy_template_by_id_and_version),
            )
            .route(
                routes::policy_templates::BY_ID_AND_VERSION,
                delete(Self::handle_delete_policy_template_by_id_and_version),
            )
            .route(
                routes::policy_templates::INSTANTIATE_ODRL_OFFER,
                post(Self::handle_instantiate_offer),
            )
            .with_state(self)
    }

    async fn handle_get_all_policy_templates(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        Query(query): Query<PolicyTemplateQuery>,
    ) -> impl IntoResponse {
        match state
            .ctx
            .templates()
            .get_all_policy_templates(&user, &query.filter, &query.page, &query.sort)
            .await
        {
            Ok(templates) => (StatusCode::OK, Json(ToCamelCase(templates))).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_get_batch_policy_templates(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        input: Result<Json<BatchRequestsAsString>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state
            .ctx
            .templates()
            .get_batch_policy_templates(&user, &input.ids)
            .await
        {
            Ok(templates) => (StatusCode::OK, Json(ToCamelCase(templates))).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_get_policy_template_by_id(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match state.ctx.templates().get_policies_template_by_id(&user, &id).await {
            Ok(templates) => (StatusCode::OK, Json(ToCamelCase(templates))).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_get_policy_template_by_id_and_version(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        Path((id, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state
            .ctx
            .templates()
            .get_policies_template_by_version_and_id(&user, &id, &version)
            .await
        {
            Ok(template) => (StatusCode::OK, Json(ToCamelCase(template))).into_response(),
            Err(e) => e.into_response(),
        }
    }
    async fn handle_create_policy_template(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        Query(params): Query<SilentParams>,
        input: Result<Json<NewPolicyTemplateDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let silent = params.silent.unwrap_or(false);
        let input = match input {
            Ok(Json(v)) => v,
            Err(e) => {
                if silent {
                    tracing::warn!("Silent mode: Invalid JSON payload ignored: {}", e);
                    return (StatusCode::OK).into_response();
                }
                return (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")).into_response();
            }
        };
        match state.ctx.templates().create_policy_template(&user, &input).await {
            Ok(template) => (StatusCode::OK, Json(ToCamelCase(template))).into_response(),
            Err(Errors::DatabaseError { reason, .. })
                if silent && reason.contains("duplicate key") =>
            {
                tracing::warn!(
                    "Policy template '{}' v{} already exists, skipping (silent mode)",
                    input.id.as_deref().unwrap_or("unknown"),
                    input.version.as_deref().unwrap_or("unknown")
                );
                StatusCode::OK.into_response()
            }
            Err(e) => e.into_response(),
        }
    }
    async fn handle_delete_policy_template_by_id_and_version(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        Path((id, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state
            .ctx
            .templates()
            .delete_policy_template_by_version_and_id(&user, &id, &version)
            .await
        {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_instantiate_offer(
        State(state): State<PolicyTemplateEntityRouter>,
        user: ymir::types::oauth::UserInfo,
        input: Result<Json<NewPolicyInstantiationDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.instantiation().instantiate_policy(&user, &input).await {
            Ok(offer) => (StatusCode::ACCEPTED, Json(OdrlPolicyView::assemble(offer))).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
