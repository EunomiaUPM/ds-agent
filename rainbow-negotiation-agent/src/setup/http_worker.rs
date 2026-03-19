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

use crate::data::factory_sql::NegotiationAgentRepoForSql;
use crate::entities::agreement::agreement::NegotiationAgentAgreementsService;
use crate::entities::negotiation_message::negotiation_message::NegotiationAgentMessagesService;
use crate::entities::negotiation_process::negotiation_process::NegotiationAgentProcessesService;
use crate::entities::offer::offer::NegotiationAgentOffersService;
use crate::http::agreement::NegotiationAgentAgreementsRouter;
use crate::http::negotiation_message::NegotiationAgentMessagesRouter;
use crate::http::negotiation_process::NegotiationAgentProcessesRouter;
use crate::http::offer::NegotiationAgentOffersRouter;
use crate::protocols::dsp::NegotiationDSP;
use crate::protocols::protocol::ProtocolPluginTrait;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::{Router, serve};
use rainbow_common::config::services::ContractsConfig;
use rainbow_common::config::types::traits::CommonConfigTrait;
use rainbow_common::errors::CommonErrors;
use rainbow_common::well_known::WellKnownRoot;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;
use ymir::config::traits::{ApiConfigTrait, ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::http::HealthRouter;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

pub struct NegotiationHttpWorker {}
impl NegotiationHttpWorker {
    pub async fn spawn(
        config: &ContractsConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> anyhow::Result<JoinHandle<()>> {
        // well known router
        let well_known_router = WellKnownRoot::get_well_known_router(&config.into())?;
        let health_router = HealthRouter::new().router();
        // module transfer router
        let router = Self::create_root_http_router(&config, vault.clone())
            .await?
            .merge(well_known_router)
            .merge(health_router);
        let host = if config.common().is_local() { "127.0.0.1" } else { "0.0.0.0" };
        let port = config.common().get_weird_port(HostType::Http);
        let addr = format!("{}{}", host, port);

        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("HTTP Negotiation Service running on {}", addr);

        let token = token.clone();
        let handle = tokio::spawn(async move {
            let server = serve(listener, router).with_graceful_shutdown(async move {
                token.cancelled().await;
                tracing::info!("HTTP Service received shutdown signal, draining connections...");
            });
            match server.await {
                Ok(_) => tracing::info!("HTTP Service stopped successfully"),
                Err(e) => tracing::error!("HTTP Service crashed: {}", e),
            }
        });

        Ok(handle)
    }
    pub async fn create_root_http_router(
        config: &ContractsConfig,
        vault: Arc<VaultService>,
    ) -> anyhow::Result<Router> {
        let router =
            create_root_http_router(config, vault.clone()).await.fallback(Self::handler_404).layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        |_req: &Request<_>| tracing::info_span!("request", id = %Uuid::new_v4()),
                    )
                    .on_request(|request: &Request<_>, _span: &tracing::Span| {
                        tracing::info!("{} {}", request.method(), request.uri());
                    })
                    .on_response(DefaultOnResponse::new().level(tracing::Level::TRACE)),
            );
        Ok(router)
    }
    async fn handler_404(uri: axum::http::Uri) -> impl IntoResponse {
        let err = CommonErrors::missing_resource_new(
            &uri.to_string(),
            "Route not found or Method not allowed",
        );
        tracing::info!("404 Not Found: {}", uri);
        err.into_response()
    }
}

pub async fn create_root_http_router(config: &ContractsConfig, vault: Arc<VaultService>) -> Router {
    // ROOT Dependency Injection
    let db_connection = vault.get_db_connection(config.common()).await;
    let config = Arc::new(config.clone());
    let negotiation_repo = Arc::new(NegotiationAgentRepoForSql::create_repo(db_connection.clone()));

    // entities
    let messages_controller_service =
        Arc::new(NegotiationAgentMessagesService::new(negotiation_repo.clone()));
    let messages_router =
        NegotiationAgentMessagesRouter::new(messages_controller_service.clone(), config.clone());
    let entities_controller_service =
        Arc::new(NegotiationAgentProcessesService::new(negotiation_repo.clone()));
    let entities_router =
        NegotiationAgentProcessesRouter::new(entities_controller_service.clone(), config.clone());
    let offer_controller_service =
        Arc::new(NegotiationAgentOffersService::new(negotiation_repo.clone()));
    let offer_router =
        NegotiationAgentOffersRouter::new(offer_controller_service.clone(), config.clone());
    let agreement_controller_service =
        Arc::new(NegotiationAgentAgreementsService::new(negotiation_repo.clone()));
    let agreement_router =
        NegotiationAgentAgreementsRouter::new(agreement_controller_service.clone(), config.clone());

    use rainbow_common::facades::ssi_auth_facade::mates_facade::MatesFacadeService;
    use rainbow_common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
    use rainbow_common::http_client::HttpClient;

    // dsp
    let http_client = Arc::new(HttpClient::new(10, 10));
    let ssi_auth_config = Arc::new(config.ssi_auth());

    let ssi_auth_service = Arc::new(SSIAuthFacadeService::new(
        ssi_auth_config.clone(),
        http_client.clone(),
    ));
    let mates_service =
        Arc::new(MatesFacadeService::new(ssi_auth_config.clone(), http_client.clone()));

    let dsp_router = NegotiationDSP::new(
        entities_controller_service.clone(),
        messages_controller_service.clone(),
        offer_controller_service.clone(),
        agreement_controller_service.clone(),
        config.clone(),
        ssi_auth_service,
        mates_service,
    )
    .build_router()
    .await
    .expect("Failed to build DSP router");

    // router
    let router_str = format!("{}/negotiation-agent", config.common().get_api_version());
    let router = Router::new()
        .nest(
            format!("{}/negotiation-messages", router_str.as_str()).as_str(),
            messages_router.router(),
        )
        .nest(
            format!("{}/negotiation-processes", router_str.as_str()).as_str(),
            entities_router.router(),
        )
        .nest(
            format!("{}/offers", router_str.as_str()).as_str(),
            offer_router.router(),
        )
        .nest(
            format!("{}/agreements", router_str.as_str()).as_str(),
            agreement_router.router(),
        )
        .nest("/dsp/current/negotiations", dsp_router);

    router
}
