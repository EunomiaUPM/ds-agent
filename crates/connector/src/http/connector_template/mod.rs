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

#![allow(unused)]
use crate::entities::connector_template::{ConnectorTemplateDto, ConnectorTemplateEntitiesTrait};
use crate::entities::filters::ConnectorTemplateFilter;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::auth::AccessScope;
use common::config::services::CatalogConfig;
use common::errors::CommonErrors;
use common::query::QuerySpec;
use serde::Deserialize;
use std::sync::Arc;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct ConnectorTemplateRouter {
    service: Arc<dyn ConnectorTemplateEntitiesTrait>,
    config: Arc<CatalogConfig>,
}

pub use common::paginated_spec::PaginationParams;
pub type ConnectorTemplateQuery = QuerySpec<ConnectorTemplateFilter>;

impl FromRef<ConnectorTemplateRouter> for Arc<dyn ConnectorTemplateEntitiesTrait> {
    fn from_ref(state: &ConnectorTemplateRouter) -> Self {
        state.service.clone()
    }
}

impl ConnectorTemplateRouter {
    pub fn new(
        service: Arc<dyn ConnectorTemplateEntitiesTrait>,
        config: Arc<CatalogConfig>,
    ) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_templates))
            .route("/", post(Self::handle_create_template))
            .route("/{id}", get(Self::handle_get_templates_by_id))
            .route(
                "/{name}/{version}",
                get(Self::handle_get_template_by_name_and_version),
            )
            .route(
                "/{name}/{version}",
                delete(Self::handle_delete_template_by_name_and_version),
            )
            .with_state(self)
    }

    async fn handle_get_all_templates(
        State(state): State<ConnectorTemplateRouter>,
        scope: AccessScope,
        Query(query): Query<ConnectorTemplateQuery>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_all_templates(&scope, &query.filter, &query.page, query.sort)
            .await
        {
            Ok(templates) => (StatusCode::OK, Json(templates)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_create_template(
        State(state): State<ConnectorTemplateRouter>,
        scope: AccessScope,
        input: Result<Json<ConnectorTemplateDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let mut input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_template(&scope, &mut input).await {
            Ok(template) => (StatusCode::OK, Json(template)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_templates_by_id(
        State(state): State<ConnectorTemplateRouter>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match state.service.get_templates_by_id(&scope, &id).await {
            Ok(templates) => (StatusCode::OK, Json(templates)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_template_by_name_and_version(
        State(state): State<ConnectorTemplateRouter>,
        scope: AccessScope,
        Path((name, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_template_by_name_and_version(&scope, &name, &version)
            .await
        {
            Ok(Some(template)) => (StatusCode::OK, Json(template)).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new("template", "Template not found");
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }

    async fn handle_delete_template_by_name_and_version(
        State(state): State<ConnectorTemplateRouter>,
        scope: AccessScope,
        Path((name, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state
            .service
            .delete_template_by_name_and_version(&scope, &name, &version)
            .await
        {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
