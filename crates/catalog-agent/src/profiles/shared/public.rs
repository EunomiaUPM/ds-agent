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

//! The public read routes of a profile: the main catalog, any resource and the profile's context,
//! with no authentication, showing only what is not private.

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use common::oauth::OwnerScope;
use serde_json::Value;
use sha2::{Digest, Sha256};
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::RolePath;

use crate::cache::ProjectionKey;
use crate::profiles::ProfileAdapterTrait;
use crate::setup::context::AppContext;

/// Who reads the public routes: nobody in particular, so only records that are not private.
pub struct PublicReader;

impl PublicReader {
    const ID: &'static str = "urn:eunomia:public-reader";
    const ROLE: &'static str = "/public";

    /// A scope admitting what is visible to everyone and owned by nobody it could act for.
    pub fn scope() -> OwnerScope {
        OwnerScope::Seeing {
            user_id: Self::ID.to_string(),
            role: RolePath::from_str(Self::ROLE).expect("a valid role path"),
        }
    }
}

/// One profile's public routes.
#[derive(Clone)]
pub struct PublicProfileRouter {
    ctx: Arc<AppContext>,
    adapter: Arc<dyn ProfileAdapterTrait>,
}

impl PublicProfileRouter {
    pub const CATALOG: &'static str = "/catalog";
    pub const RESOURCE: &'static str = "/resources/{id}";
    pub const CONTEXT: &'static str = "/context.jsonld";

    pub fn new(ctx: Arc<AppContext>, adapter: Arc<dyn ProfileAdapterTrait>) -> Self {
        Self { ctx, adapter }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(Self::CATALOG, get(Self::handle_catalog))
            .route(Self::RESOURCE, get(Self::handle_resource))
            .route(Self::CONTEXT, get(Self::handle_context))
            .with_state(self)
    }

    async fn handle_catalog(State(state): State<Self>, headers: HeaderMap) -> Response {
        match state.catalog().await {
            Ok(body) => state.respond(&headers, body),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_resource(
        State(state): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
    ) -> Response {
        let Ok(id) = Urn::from_str(&id) else {
            return Errors::missing_resource(id, "resource not found", None).into_response();
        };
        match state.resource(&id).await {
            Ok(body) => state.respond(&headers, body),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_context(State(state): State<Self>) -> Response {
        let mut response = Json(state.adapter.context()).into_response();
        Self::json_ld(&mut response);
        response
    }

    /// The main catalog projected, from the cache when this generation already has it.
    async fn catalog(&self) -> Outcome<Value> {
        let key = self.key("main").await;
        if let Some(cached) = self.ctx.cache.get_projection(&key).await {
            return Ok(cached);
        }
        let (root, set) = self
            .ctx
            .closures()
            .main_catalog(&PublicReader::scope())
            .await?;
        let body = self.adapter.project(&set.published(), &root)?;
        self.ctx.cache.put_projection(&key, &body).await;
        Ok(body)
    }

    async fn resource(&self, id: &Urn) -> Outcome<Value> {
        let key = self.key(id.as_ref()).await;
        if let Some(cached) = self.ctx.cache.get_projection(&key).await {
            return Ok(cached);
        }
        let (root, set) = self
            .ctx
            .closures()
            .resource(&PublicReader::scope(), id)
            .await?
            .ok_or_else(|| Errors::missing_resource(id.to_string(), "resource not found", None))?;
        // A draft is not shared: to the public it does not exist.
        let set = set
            .shared_from(&root)
            .ok_or_else(|| Errors::missing_resource(id.to_string(), "resource not found", None))?;
        let body = self.adapter.project(&set, &root)?;
        self.ctx.cache.put_projection(&key, &body).await;
        Ok(body)
    }

    async fn key(&self, root: &str) -> String {
        let generation = self.ctx.cache.generation().await;
        ProjectionKey::of(self.adapter.key(), root, "public", generation)
    }

    /// JSON-LD with the profile link, a strong tag over the body and `304` when it matches.
    fn respond(&self, headers: &HeaderMap, body: Value) -> Response {
        let etag = format!(
            "\"{}\"",
            hex::encode(&Sha256::digest(body.to_string().as_bytes())[..12])
        );
        if headers
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|tags| tags.split(',').any(|t| t.trim() == etag))
        {
            return StatusCode::NOT_MODIFIED.into_response();
        }
        let mut response = Json(body).into_response();
        Self::json_ld(&mut response);
        let headers = response.headers_mut();
        if let Ok(link) = HeaderValue::from_str(&format!(
            "<{}>; rel=\"profile\"",
            self.adapter.profile_iri()
        )) {
            headers.insert(header::LINK, link);
        }
        if let Ok(tag) = HeaderValue::from_str(&etag) {
            headers.insert(header::ETAG, tag);
        }
        headers.insert(header::VARY, HeaderValue::from_static("Accept"));
        response
    }

    fn json_ld(response: &mut Response) {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/ld+json"),
        );
    }
}
