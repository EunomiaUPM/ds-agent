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

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::Router;
use common::auth::http::AuthHttpMiddleware;
use common::auth::OauthTokenValidator;
use common::config::types::traits::CommonConfigTrait;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};
use uuid::Uuid;
use ymir::config::traits::ApiConfigTrait;
use ymir::http::{HealthRouter, OpenapiRouter, WalletRouter};

use crate::core::AuthCore;
use crate::http::gatekeeper_router::GateKeeperRouter;
use crate::http::peer_connector_router::OnboarderRouter;
use crate::http::verifier_router::VerifierRouter;
use crate::http::{GaiaSelfAttesterRouter, ParticipantRouter, VcRequesterRouter};
use crate::services::HasConfig;

pub struct AuthRouter {
    core: Arc<AuthCore>,
    openapi: String,
    validator: Arc<dyn OauthTokenValidator>,
}

impl AuthRouter {
    pub fn new(core: Arc<AuthCore>, validator: Arc<dyn OauthTokenValidator>) -> Self {
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

    pub fn router(self) -> Router {
        let wallet_router = WalletRouter::new(self.core.clone());
        let vc_requester_router = VcRequesterRouter::new(self.core.clone());
        let gatekeeper_router = GateKeeperRouter::new(self.core.clone());
        let mate_router = ParticipantRouter::new(self.core.clone());
        let verifier_router = VerifierRouter::new(self.core.clone());
        let onboarder_router = OnboarderRouter::new(self.core.clone());
        let gaia_router = GaiaSelfAttesterRouter::new(self.core.clone());
        let openapi_router = OpenapiRouter::new(self.openapi.clone());
        let health_router = HealthRouter::new();

        let api_path = self.core.config().common().get_api_version();

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
        let guard = from_fn_with_state(self.validator.clone(), AuthHttpMiddleware::run);

        let router = Router::new()
            .merge(wallet_router.well_known())
            // .merge(gaia_router.well_known())
            .nest(
                &format!("{}/wallet", api_path),
                wallet_router.router().route_layer(guard.clone()),
            )
            .nest(
                &format!("{}/mates", api_path),
                mate_router.router().route_layer(guard.clone()),
            )
            .nest(&format!("{}", api_path), health_router.router())
            .nest(
                &format!("{}/vc-request", api_path),
                vc_requester_router
                    .protocol_router()
                    .merge(vc_requester_router.router().route_layer(guard.clone())),
            )
            .nest(
                &format!("{}/gate", api_path),
                gatekeeper_router
                    .protocol_router()
                    .merge(gatekeeper_router.router().route_layer(guard.clone())),
            )
            .nest(&format!("{}/verifier", api_path), verifier_router.router())
            .nest(
                &format!("{}/peer-connection", api_path),
                onboarder_router
                    .protocol_router()
                    .merge(onboarder_router.router().route_layer(guard.clone())),
            )
            .nest(
                &format!("{}/gaia", api_path),
                gaia_router.router().route_layer(guard),
            )
            .nest(&format!("{}/docs", api_path), openapi_router.router());

        router.fallback(Self::fallback).layer(cors).layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    |_req: &Request<_>| tracing::info_span!("Auth-request", id = %Uuid::new_v4()),
                )
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
