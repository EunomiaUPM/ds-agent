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

//! The management API of one kind of resource: JSON-LD in the catalog context in and out,
//! version, visibility and owner in headers, and the doctor of each resource.

pub mod headers;

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::query::QuerySpec;
use common::routes::catalog as routes;
use serde::Deserialize;
use serde_json::Value;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::oauth::UserInfo;
use ymir::utils::extract_payload;

use crate::entities::filters::ResourceFilter;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::{Parent, ResourceKind};
use crate::http::resources::headers::AdminHeaders;
use crate::setup::context::AppContext;

pub type ResourceQuery = QuerySpec<ResourceFilter>;

/// What the doctor route takes.
#[derive(Debug, Default, Deserialize)]
pub struct DoctorQuery {
    pub profile: Option<String>,
    #[serde(default)]
    pub deep: bool,
}

/// The routes of one kind of resource.
#[derive(Clone)]
pub struct ResourceRouter {
    ctx: Arc<AppContext>,
    kind: ResourceKind,
}

impl ResourceRouter {
    pub fn new(ctx: Arc<AppContext>, kind: ResourceKind) -> Self {
        Self { ctx, kind }
    }

    pub fn router(self) -> Router {
        let mut router = Router::new()
            .route(routes::entity::ROOT, get(Self::handle_list).post(Self::handle_create))
            .route(routes::entity::BATCH, axum::routing::post(Self::handle_batch))
            .route(
                routes::entity::BY_ID,
                get(Self::handle_get).put(Self::handle_put).delete(Self::handle_delete),
            )
            .route(routes::entity::DOCTOR, get(Self::handle_doctor));
        if self.kind.has_main() {
            router = router.route(
                routes::entity::MAIN,
                get(Self::handle_get_main).post(Self::handle_create_main),
            );
        }
        if self.kind.parent() == Parent::Catalog {
            router = router.route(routes::entity::BY_CATALOG, get(Self::handle_children));
        }
        if self.kind == ResourceKind::Distribution {
            router = router
                .route(routes::distributions::BY_DATASET, get(Self::handle_children))
                .route(routes::distributions::BY_DATASET_AND_FORMAT, get(Self::handle_by_format));
        }
        router.with_state(self)
    }

    async fn handle_list(
        State(state): State<Self>,
        user: UserInfo,
        Query(query): Query<ResourceQuery>,
    ) -> Response {
        let (mut filter, page, sort) = query.into_domain();
        filter.kind = Some(state.kind);
        match state.ctx.resources().list(&user, &filter, &page, &sort).await {
            Ok(page) => AdminHeaders::page(&page),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_batch(
        State(state): State<Self>,
        user: UserInfo,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(input) => input,
            Err(e) => return e.into_response(),
        };
        match state.ctx.resources().batch(&user, state.kind, &input.ids).await {
            Ok(found) => AdminHeaders::graph(&found),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_get(State(state): State<Self>, user: UserInfo, Path(id): Path<String>) -> Response {
        let id = match Self::urn(&id) {
            Ok(id) => id,
            Err(e) => return e.into_response(),
        };
        match state.ctx.resources().get(&user, state.kind, &id).await {
            Ok(resource) => AdminHeaders::resource(StatusCode::OK, &resource),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_get_main(State(state): State<Self>) -> Response {
        match state.ctx.resources().main(state.kind).await {
            Ok(Some(resource)) => AdminHeaders::resource(StatusCode::OK, &resource),
            Ok(None) => Errors::missing_resource(
                "main",
                format!("the connector has no main {}", state.kind.term()),
                None,
            )
            .into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_children(State(state): State<Self>, user: UserInfo, Path(id): Path<String>) -> Response {
        let parent = match Self::urn(&id) {
            Ok(id) => id,
            Err(e) => return e.into_response(),
        };
        match state.ctx.resources().children(&user, state.kind, &parent).await {
            Ok(children) => AdminHeaders::graph(&children),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_by_format(
        State(state): State<Self>,
        user: UserInfo,
        Path((id, format)): Path<(String, String)>,
    ) -> Response {
        let dataset = match Self::urn(&id) {
            Ok(id) => id,
            Err(e) => return e.into_response(),
        };
        match state.ctx.resources().distribution_by_format(&user, &dataset, &format).await {
            Ok(distribution) => AdminHeaders::resource(StatusCode::OK, &distribution),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_create(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        input: Result<Json<Value>, JsonRejection>,
    ) -> Response {
        match state.create(&user, &headers, input, false).await {
            Ok(resource) => AdminHeaders::resource(StatusCode::CREATED, &resource),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_create_main(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        input: Result<Json<Value>, JsonRejection>,
    ) -> Response {
        match state.create(&user, &headers, input, true).await {
            Ok(resource) => AdminHeaders::resource(StatusCode::CREATED, &resource),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_put(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        Path(id): Path<String>,
        input: Result<Json<Value>, JsonRejection>,
    ) -> Response {
        match state.replace(&user, &headers, &id, input).await {
            Ok(resource) => AdminHeaders::resource(StatusCode::OK, &resource),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_delete(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        Path(id): Path<String>,
    ) -> Response {
        let outcome = async {
            let id = Self::urn(&id)?;
            let expected = AdminHeaders::expected_version(&headers)?;
            state.ctx.resources().delete(&user, state.kind, &id, expected).await
        };
        match outcome.await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_doctor(
        State(state): State<Self>,
        user: UserInfo,
        Path(id): Path<String>,
        Query(query): Query<DoctorQuery>,
    ) -> Response {
        let id = match Self::urn(&id) {
            Ok(id) => id,
            Err(e) => return e.into_response(),
        };
        match state
            .ctx
            .doctor()
            .diagnose(&user, state.kind, &id, query.profile, query.deep)
            .await
        {
            Ok(report) => Json(report).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn create(
        &self,
        user: &UserInfo,
        headers: &HeaderMap,
        input: Result<Json<Value>, JsonRejection>,
        main: bool,
    ) -> Outcome<CatalogResource> {
        let body = extract_payload(input)?;
        let visibility = AdminHeaders::visibility(headers)?;
        self.ctx
            .resources()
            .create_document(user, self.kind, body, visibility, main)
            .await
    }

    async fn replace(
        &self,
        user: &UserInfo,
        headers: &HeaderMap,
        id: &str,
        input: Result<Json<Value>, JsonRejection>,
    ) -> Outcome<CatalogResource> {
        let id = Self::urn(id)?;
        let body = extract_payload(input)?;
        let visibility = AdminHeaders::visibility(headers)?;
        let expected = AdminHeaders::expected_version(headers)?;
        self.ctx
            .resources()
            .replace_document(user, self.kind, &id, body, visibility, expected)
            .await
    }

    fn urn(id: &str) -> Outcome<Urn> {
        Urn::from_str(id).map_err(|_| Errors::format(BadFormat::Received, format!("{id} is not a URN"), None))
    }
}
