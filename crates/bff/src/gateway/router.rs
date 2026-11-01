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

use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::middleware::from_fn_with_state;
use axum::response::Response;
use axum::routing::{any, get};
use axum::Router;
use ymir::http::OauthHttpMiddleware;
use tower_http::cors::{Any, CorsLayer};

use crate::gateway::discovery::DiscoveryHandlers;
use crate::gateway::frontend::FrontendHandlers;
use crate::gateway::live::LiveHandlers;
use crate::gateway::session::SessionHandlers;
use crate::setup::context::AppContext;
use common::routes::bff as routes;

/// Gateway routes. Proxied calls are authenticated by the agents they reach.
#[derive(Clone)]
pub struct GatewayHttpRouter {
    ctx: Arc<AppContext>,
}

impl GatewayHttpRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn console_router(self) -> Router {
        Router::new()
            .route(routes::ROOT, get(FrontendHandlers::console_root))
            .route(routes::CONSOLE_INDEX, get(FrontendHandlers::static_asset))
            .nest(routes::CONSOLE, self.router())
    }

    /// The API answers under both `/admin/api` and `/api`; any other path serves the SPA.
    pub fn router(self) -> Router {
        // Exposed headers too: the dev console reads ETag, Eunomia-* and paging across origins.
        let cors = CorsLayer::new()
            .allow_methods(Any)
            .allow_origin(Any)
            .allow_headers(Any)
            .expose_headers(Any);
        let api = self.api_router();
        Router::new()
            .nest(routes::CONSOLE_API, api.clone())
            .nest(routes::API, api)
            .fallback(FrontendHandlers::static_asset)
            .layer(cors)
    }

    fn api_router(&self) -> Router {
        Router::new()
            .route(routes::FE_CONFIG, get(FrontendHandlers::fe_config))
            .route(
                routes::DSP,
                any(Self::proxy_dsp),
            )
            .route(routes::WELL_KNOWN_RPC, any(Self::proxy_well_known_rpc))
            .route(routes::SERVICE, any(Self::proxy))
            .route(routes::SERVICE_PATH, any(Self::proxy_with_extra))
            .merge(self.guarded_router())
            .with_state(self.ctx.clone())
    }

    /// Routes for authenticated callers only: who the session is, the live updates, and
    /// discovery, which fetches arbitrary URLs.
    fn guarded_router(&self) -> Router<Arc<AppContext>> {
        let Some(validator) = self.ctx.oauth_validator.clone() else {
            return Router::new();
        };
        Router::new()
            .route(routes::ME, get(SessionHandlers::me))
            .route(routes::LIVE, get(LiveHandlers::live))
            .route(routes::DID_JSON, get(DiscoveryHandlers::did_json))
            .route(
                routes::FEDERATED_CATALOG,
                get(DiscoveryHandlers::federated_catalog),
            )
            .route_layer(from_fn_with_state(validator, OauthHttpMiddleware::run))
    }

    async fn proxy(
        State(ctx): State<Arc<AppContext>>,
        Path(service_prefix): Path<String>,
        req: Request<Body>,
    ) -> Response {
        ctx.proxy.proxy_request(service_prefix, None, req).await
    }

    async fn proxy_with_extra(
        State(ctx): State<Arc<AppContext>>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> Response {
        ctx.proxy
            .proxy_request(service_prefix, Some(extra), req)
            .await
    }

    async fn proxy_dsp(
        State(ctx): State<Arc<AppContext>>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> Response {
        ctx.proxy
            .proxy_dsp_request(service_prefix, Some(extra), req)
            .await
    }

    async fn proxy_well_known_rpc(
        State(ctx): State<Arc<AppContext>>,
        Path(extra): Path<String>,
        req: Request<Body>,
    ) -> Response {
        ctx.proxy.proxy_well_known_rpc_request(extra, req).await
    }
}
