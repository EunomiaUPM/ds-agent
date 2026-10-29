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

//! The schema API: hosted schemas and their versions, inference, checks; and the public route
//! where anyone dereferences a version.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::json_schema::JSON_SCHEMA_MEDIA_TYPE;
use common::query::QuerySpec;
use common::routes::catalog as routes;
use ymir::errors::{BadFormat, Errors};
use ymir::types::oauth::UserInfo;
use ymir::utils::extract_payload;

use crate::entities::filters::SchemaFilter;
use crate::entities::schemas::{
    CheckSchemaUrlDto, InferSchemaDto, NewSchemaDto, NewSchemaVersionDto, ValidateSchemaDto,
};
use crate::http::common::to_camel_case::ToCamelCase;
use crate::setup::context::AppContext;

pub type SchemaQuery = QuerySpec<SchemaFilter>;

/// Management routes, behind the admin authentication.
#[derive(Clone)]
pub struct SchemaRouter {
    ctx: Arc<AppContext>,
}

impl SchemaRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                routes::schemas::ROOT,
                get(Self::handle_list).post(Self::handle_create),
            )
            .route(routes::schemas::INFER, post(Self::handle_infer))
            .route(routes::schemas::VALIDATE, post(Self::handle_validate))
            .route(routes::schemas::CHECK_URL, post(Self::handle_check_url))
            .route(
                routes::schemas::BY_ID,
                get(Self::handle_versions).delete(Self::handle_delete_schema),
            )
            .route(routes::schemas::VERSIONS, post(Self::handle_add_version))
            .route(
                routes::schemas::BY_ID_AND_VERSION,
                get(Self::handle_version).delete(Self::handle_delete_version),
            )
            .with_state(self)
    }

    async fn handle_list(
        State(state): State<Self>,
        user: UserInfo,
        Query(query): Query<SchemaQuery>,
    ) -> Response {
        match state
            .ctx
            .schemas()
            .list(&user, &query.filter, &query.page, &query.sort)
            .await
        {
            Ok(page) => Json(ToCamelCase(page)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_create(
        State(state): State<Self>,
        user: UserInfo,
        input: Result<Json<NewSchemaDto>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().create(&user, &input).await {
            Ok(view) => (StatusCode::CREATED, Json(view)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_versions(
        State(state): State<Self>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> Response {
        match state.ctx.schemas().versions(&user, &id).await {
            Ok(views) => Json(views).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_add_version(
        State(state): State<Self>,
        user: UserInfo,
        Path(id): Path<String>,
        input: Result<Json<NewSchemaVersionDto>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().add_version(&user, &id, &input).await {
            Ok(view) => (StatusCode::CREATED, Json(view)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_version(
        State(state): State<Self>,
        user: UserInfo,
        Path((id, version)): Path<(String, String)>,
    ) -> Response {
        let version = match Self::version_number(&version) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().version(&user, &id, version).await {
            Ok(view) => Json(view).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_delete_schema(
        State(state): State<Self>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> Response {
        match state.ctx.schemas().delete(&user, &id, None).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_delete_version(
        State(state): State<Self>,
        user: UserInfo,
        Path((id, version)): Path<(String, String)>,
    ) -> Response {
        let version = match Self::version_number(&version) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().delete(&user, &id, Some(version)).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_infer(
        State(state): State<Self>,
        _user: UserInfo,
        input: Result<Json<InferSchemaDto>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().infer(&input) {
            Ok(schema) => Json(schema).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_validate(
        State(state): State<Self>,
        _user: UserInfo,
        input: Result<Json<ValidateSchemaDto>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().validate(&input) {
            Ok(report) => Json(report).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_check_url(
        State(state): State<Self>,
        _user: UserInfo,
        input: Result<Json<CheckSchemaUrlDto>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().check_url(&input).await {
            Ok(report) => Json(report).into_response(),
            Err(e) => e.into_response(),
        }
    }

    /// `3` or `v3`.
    fn version_number(raw: &str) -> Result<i32, Errors> {
        raw.trim_start_matches('v').parse().map_err(|_| {
            Errors::format(
                BadFormat::Received,
                format!("{raw} is not a version number"),
                None,
            )
        })
    }
}

/// The public route: a version as `application/schema+json`, to anyone, never changing.
#[derive(Clone)]
pub struct PublicSchemaRouter {
    ctx: Arc<AppContext>,
}

impl PublicSchemaRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::public::SCHEMA, get(Self::handle_schema))
            .with_state(self)
    }

    async fn handle_schema(
        State(state): State<Self>,
        Path((id, version)): Path<(String, String)>,
    ) -> Response {
        let version = match SchemaRouter::version_number(&version) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.ctx.schemas().published(&id, version).await {
            Ok(schema) => {
                let mut response = Json(schema).into_response();
                let headers = response.headers_mut();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(JSON_SCHEMA_MEDIA_TYPE),
                );
                // A version never changes, so anyone may keep it.
                headers.insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                );
                headers.insert(
                    header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    HeaderValue::from_static("*"),
                );
                response
            }
            Err(e) => e.into_response(),
        }
    }
}
