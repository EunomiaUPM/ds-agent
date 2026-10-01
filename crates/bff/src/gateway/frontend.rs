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

//! Embedded admin SPA and the runtime configuration it bootstraps from.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::config::types::traits::CommonConfigTrait;
use rust_embed::Embed;
use serde_json::json;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

use crate::setup::context::AppContext;

#[derive(Embed)]
#[folder = "src/static/admin/dist"]
struct ReactApp;

pub(crate) struct FrontendHandlers;

impl FrontendHandlers {
    /// Serves a bundled asset, falling back to `index.html` for client-side routes.
    pub(crate) async fn static_asset(uri: Uri) -> Response {
        let path = match uri.path().trim_start_matches('/') {
            "" => "index.html",
            path => path,
        };
        let (path, content) = match ReactApp::get(path) {
            Some(content) => (path, content),
            None => match ReactApp::get("index.html") {
                Some(content) => ("index.html", content),
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        "<h1>404</h1><p>index.html not found</p>",
                    )
                        .into_response()
                }
            },
        };
        let mime_type = mime_guess::from_path(path).first_or_octet_stream();
        Response::builder()
            .header(header::CONTENT_TYPE, mime_type.as_ref())
            .body(Body::from(content.data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }

    pub(crate) async fn fe_config(State(ctx): State<Arc<AppContext>>) -> impl IntoResponse {
        let gateway_base = ctx.config.common().hosts.get_host(HostType::Http);
        Json(json!({ "gateway_base": gateway_base }))
    }
}
