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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use common::config::types::traits::CommonConfigTrait;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::{error, info, Level};
use uuid::Uuid;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::http::{HealthRouter, OpenapiRouter, WalletRouter};
use ymir::types::dids::{DidService, DidServiceType};

use crate::core::traits::AuthCoreTrait;
use crate::core::AuthCore;
use crate::http::business_router::BusinessRouter;
use crate::http::gatekeeper_router::GateKeeperRouter;
use crate::http::onboarder_router::OnboarderRouter;
use crate::http::verifier_router::VerifierRouter;
use crate::http::{GaiaSelfIssuerRouter, MateRouter, VcRequesterRouter};

pub struct AuthRouter {
    core: Arc<AuthCore>,
    openapi: String,
}

impl AuthRouter {
    pub fn new(core: Arc<AuthCore>) -> Self {
        let openapi = core
            .config()
            .common()
            .get_openapi()
            .expect("Invalid openapi path");
        AuthRouter { core, openapi }
    }

    pub fn router(self) -> Router {
        let vc_requester_router = VcRequesterRouter::new(self.core.clone());
        let gatekeeper_router = GateKeeperRouter::new(self.core.clone());
        let mate_router = MateRouter::new(self.core.clone());
        let verifier_router = VerifierRouter::new(self.core.clone());
        let openapi_router = OpenapiRouter::new(self.openapi.clone());
        let business_router = BusinessRouter::new(self.core.clone());
        let onboarder_router = OnboarderRouter::new(self.core.clone());
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

        let router = Router::new()
            .nest(&format!("{}", api_path), health_router.router())
            .nest(&format!("{}/mates", api_path), mate_router.router())
            .nest(
                &format!("{}/vc-request", api_path),
                vc_requester_router.router(),
            )
            .nest(&format!("{}/gate", api_path), gatekeeper_router.router())
            .nest(&format!("{}/verifier", api_path), verifier_router.router())
            .nest(&format!("{}/business", api_path), business_router.router())
            .nest(&format!("{}/onboard", api_path), onboarder_router.router())
            .nest(&format!("{}/docs", api_path), openapi_router.router());

        let router = match self.core.is_gaia_active() {
            true => {
                let gaia_router = GaiaSelfIssuerRouter::new(self.core.clone());
                router
                    .merge(gaia_router.well_known())
                    .nest(&format!("{}/gaia", api_path), gaia_router.router())
            }
            false => router,
        };

        let router = match self.core.is_wallet_active() {
            true => {
                let services = vec![DidService::basic(
                    DidServiceType::AuthorizationServer,
                    format!(
                        "{}{}/gate/access",
                        self.core.config().common().get_host(HostType::Http),
                        api_path
                    ),
                )];
                let wallet = WalletRouter::new(self.core.clone());
                router
                    .merge(wallet.well_known(Some(services)))
                    .nest(&format!("{}/wallet", api_path), wallet.router())
            }
            false => router,
        };

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
