/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use std::sync::Arc;

use axum::extract::Request;
use axum::Router;
use rainbow_auth::setup::app::AuthApplication;
use rainbow_catalog_agent::setup::create_root_http_router as catalog_router;
use rainbow_common::config::ApplicationConfig;
use rainbow_common::well_known::WellKnownRoot;
use rainbow_fe_gateway::create_gateway_http_router;
use rainbow_negotiation_agent::create_negotiations_http_router;
// use rainbow_transfer_agent::setup::create_root_http_router;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;
use ymir::services::vault::global::VaultService;

pub async fn create_core_router(config: &ApplicationConfig, vault: Arc<VaultService>) -> Router {
    let well_known_root_dspace =
        WellKnownRoot::get_well_known_router(&config.into()).expect("Failed to well known router");
    let auth_router = AuthApplication::create_router(&config.ssi_auth(), vault.clone()).await;
    let negotiation_agent_router =
        create_negotiations_http_router(&config.contracts(), vault.clone()).await;
    // let transfer_agent_router = create_root_http_router(&config.transfer(), vault.clone())
    //     .await
    //     .expect("Failed to create transfer agent router");
    let catalog_agent_router = catalog_router(&config.catalog(), vault.clone())
        .await
        .expect("Failed to create catalog router");
    let gateway_router = create_gateway_http_router(&config.gateway()).await;

    Router::new()
        .merge(well_known_root_dspace)
        .merge(catalog_agent_router)
        .merge(auth_router)
        .merge(negotiation_agent_router)
        // .merge(transfer_agent_router)
        .merge(gateway_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    |_req: &Request<_>| tracing::info_span!("request", id = %Uuid::new_v4())
                )
                .on_request(|request: &Request<_>, _span: &tracing::Span| {
                    tracing::info!("{} {}", request.method(), request.uri());
                })
                .on_response(DefaultOnResponse::new().level(tracing::Level::TRACE))
        )
}
