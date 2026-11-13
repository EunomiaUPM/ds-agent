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

//! Static landing page rendered on `GET /` by the monolith binary.

use axum::extract::State;
use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use common::routes::{auth, bff};
use ymir::http::routes::openapi;

const LANDING_HTML: &str = include_str!("landing.html");

/// Serves the Eunomia landing HTML linking to the admin frontend and OpenAPI docs.
#[derive(Clone)]
pub struct LandingRouter {
    admin_url: String,
    swagger_url: String,
    spec_url: String,
}

impl LandingRouter {
    /// Builds a landing router from the configured HTTP API prefix (e.g. `/api/v1`).
    pub fn for_api_base(api_base: &str) -> Self {
        let base = api_base.trim_end_matches('/');
        let docs_base = format!("{base}{}", auth::docs::PREFIX);
        Self {
            admin_url: bff::CONSOLE_INDEX.to_string(),
            swagger_url: format!("{docs_base}{}", openapi::SWAGGER),
            spec_url: format!("{docs_base}{}", openapi::JSON),
        }
    }

    /// Mounts `GET /` for the monolith landing page.
    pub fn router(self) -> Router {
        Router::new()
            .route(bff::ROOT, get(Self::handle_root))
            .with_state(self)
    }

    /// Renders the static HTML document with configured entrypoint URLs.
    pub fn render_html(&self) -> String {
        LANDING_HTML
            .replace("__VERSION__", &Self::escape_html(env!("CARGO_PKG_VERSION")))
            .replace("__ADMIN_URL__", &Self::escape_html(&self.admin_url))
            .replace("__SWAGGER_URL__", &Self::escape_html(&self.swagger_url))
            .replace("__SPEC_URL__", &Self::escape_html(&self.spec_url))
    }

    async fn handle_root(State(router): State<Self>) -> impl IntoResponse {
        (
            [(header::CACHE_CONTROL, "no-store")],
            Html(router.render_html()),
        )
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}
