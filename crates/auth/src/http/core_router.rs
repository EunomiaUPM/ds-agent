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

use crate::core::AuthCore;
use crate::http::gatekeeper_router::GateKeeperRouter;
use crate::http::peer_connector_router::OnboarderRouter;
use crate::http::verifier_router::VerifierRouter;
use crate::http::{GaiaSelfAttesterRouter, ParticipantRouter, VcRequesterRouter};
use crate::services::HasConfig;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::Router;
use common::config::types::traits::CommonConfigTrait;
use common::oauth::token_validator;
use common::routes::auth::{docs, gaia, gate, mates, peer_connection, vc_request, wallet};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};
use uuid::Uuid;
use ymir::config::traits::ApiConfigTrait;
use ymir::errors::Outcome;
use ymir::http::routes::verifier;
use ymir::http::OauthHttpMiddleware;
use ymir::http::{HealthRouter, OpenapiRouter, WalletRouter};
use ymir::services::token_validator::OauthTokenValidatorTrait;

/// Every auth route, plus health and the OpenAPI document; panics on a bad OpenAPI path.
pub struct AuthRouter {
    core: Arc<AuthCore>,
    openapi: String,
    validator: Arc<dyn OauthTokenValidatorTrait>,
}

impl AuthRouter {
    pub fn new(core: Arc<AuthCore>, validator: Arc<dyn OauthTokenValidatorTrait>) -> Self {
        let openapi = core
            .config()
            .common()
            .get_openapi()
            .expect("Invalid openapi path");

        AuthRouter {
            core,
            openapi,
            validator,
        }
    }

    /// Every route grouped by who calls it: external (protocol routes for peers, authorities and
    /// wallets, open) and internal (user routes, all behind the OAuth guard). A capability with
    /// both nests each half under the same prefix; axum flattens nested routes, so they merge.
    pub fn router(self) -> Router {
        // ===== SUB-ROUTERS =======================================================================
        let wallet_r = WalletRouter::new(self.core.clone());
        let vc_router = VcRequesterRouter::new(self.core.clone());
        let gatekeeper_r = GateKeeperRouter::new(self.core.clone());
        let mate_r = ParticipantRouter::new(self.core.clone());
        let verifier_r = VerifierRouter::new(self.core.clone());
        let peer_r = OnboarderRouter::new(self.core.clone());
        let gaia_r = GaiaSelfAttesterRouter::new(self.core.clone());
        let openapi_r = OpenapiRouter::new(self.openapi.clone());
        let health_r = HealthRouter::new();

        let api_path = self.core.config().common().get_api_version();

        // ===== LAYERS ============================================================================
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::DELETE,
            ])
            .allow_headers(Any)
            .allow_credentials(false);

        // User routes need an OAuth token; protocol routes are authenticated by GNAP/OID4VP.
        let guard = from_fn_with_state(self.validator.clone(), OauthHttpMiddleware::run);

        // ===== WELL-KNOWN ========================================================================
        let well_known = wallet_r.well_known();

        // ===== EXTERNAL: peers, authorities and wallets, authenticated by GNAP/OID4VP ============
        let external = Router::new()
            .nest(verifier::PREFIX, verifier_r.external())
            .nest(vc_request::PREFIX, vc_router.external())
            .nest(gate::PREFIX, gatekeeper_r.external())
            .nest(peer_connection::PREFIX, peer_r.external());

        // ===== INTERNAL: users, behind the OAuth guard ===========================================
        let internal = Router::new()
            .nest(wallet::PREFIX, wallet_r.internal())
            .nest(mates::PREFIX, mate_r.internal())
            .nest(vc_request::PREFIX, vc_router.internal())
            .nest(gate::PREFIX, gatekeeper_r.internal())
            .nest(peer_connection::PREFIX, peer_r.internal())
            .nest(gaia::PREFIX, gaia_r.internal())
            .route_layer(guard);

        // ===== OPEN: health and API docs =========================================================
        let open = Router::new()
            .merge(health_r.router())
            .nest(docs::PREFIX, openapi_r.router());

        // ===== ALL TOGETHER: well-known at the root, the rest under the API version ==============
        let api = Router::new().merge(external).merge(internal).merge(open);
        
        Router::new()
            .merge(well_known)
            .nest(&api_path, api)
            .fallback(Self::fallback)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|_req: &Request<_>| {
                        tracing::info_span!("Auth-request", id = %Uuid::new_v4())
                    })
                    .on_request(|req: &Request<_>, _span: &tracing::Span| {
                        info!("{} {}", req.method(), req.uri().path());
                    })
                    .on_response(DefaultOnResponse::new().level(Level::TRACE)),
            )
    }

    async fn fallback() -> impl IntoResponse {
        error!("Wrong route");
        StatusCode::NOT_FOUND.into_response()
    }
}
