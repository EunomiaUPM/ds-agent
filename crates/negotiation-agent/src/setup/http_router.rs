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

//! Legacy HTTP plane of the negotiation agent, built as one router until it becomes a module.

use crate::data::factory_sql::NegotiationAgentRepoForSql;
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::http::agreement::NegotiationAgentAgreementsRouter;
use crate::http::negotiation_message::NegotiationAgentMessagesRouter;
use crate::http::negotiation_process::NegotiationAgentProcessesRouter;
use crate::http::offer::NegotiationAgentOffersRouter;
use crate::protocols::dsp::NegotiationDSP;
use crate::protocols::protocol::ProtocolPluginTrait;
use crate::services::agreement::service::AgreementService;
use crate::services::negotiation_message::service::NegotiationMessageService;
use crate::services::negotiation_process::service::NegotiationProcessService;
use crate::services::offer::service::OfferService;
use axum::Router;
use common::config::services::ContractsConfig;
use common::config::services::traits::ContractsConfigTrait;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use std::sync::Arc;
use ymir::config::traits::ApiConfigTrait;

pub async fn create_root_http_router(
    config: &ContractsConfig,
    root: &RootContext,
    event_bus: Option<events::EventBus>,
) -> Router {
    // ROOT Dependency Injection
    let db_connection = root.db.clone();
    let config = Arc::new(config.clone());
    let negotiation_repo = Arc::new(NegotiationAgentRepoForSql::create_repo(
        db_connection.clone(),
    ));

    // Services shared by the management API and the DSP; both emit domain events.
    let process_service = Arc::new(
        NegotiationProcessService::new(
            negotiation_repo.get_negotiation_process_repo(),
            negotiation_repo.get_negotiation_process_identifiers_repo(),
            negotiation_repo.get_negotiation_message_repo(),
            negotiation_repo.get_offer_repo(),
            negotiation_repo.get_agreement_repo(),
        )
        .with_event_bus(event_bus.clone()),
    );
    let message_service = Arc::new(
        NegotiationMessageService::new(
            negotiation_repo.get_negotiation_message_repo(),
            negotiation_repo.get_offer_repo(),
            negotiation_repo.get_agreement_repo(),
        )
        .with_event_bus(event_bus.clone()),
    );
    let offer_service = Arc::new(
        OfferService::new(negotiation_repo.get_offer_repo()).with_event_bus(event_bus.clone()),
    );
    let agreement_service = Arc::new(
        AgreementService::new(negotiation_repo.get_agreement_repo())
            .with_event_bus(event_bus.clone()),
    );

    let entities_router = NegotiationAgentProcessesRouter::new(process_service.clone());
    let messages_router = NegotiationAgentMessagesRouter::new(message_service.clone());
    let offer_router = NegotiationAgentOffersRouter::new(offer_service.clone());
    let agreement_router = NegotiationAgentAgreementsRouter::new(agreement_service.clone());

    let validator = root.validator.clone();

    use common::facades::ssi_auth_facade::mates_facade::MatesFacadeService;
    use common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;

    // dsp
    let ssi_auth_config = Arc::new(config.ssi_auth().clone());

    let service_client = root.service_client.clone();
    let ssi_auth_service = Arc::new(SSIAuthFacadeService::new(
        ssi_auth_config.clone(),
        service_client.clone(),
    ));
    let mates_service = Arc::new(MatesFacadeService::new(
        ssi_auth_config.clone(),
        service_client,
    ));

    let dsp_router = NegotiationDSP::new(
        negotiation_repo.get_negotiation_process_repo(),
        process_service,
        message_service,
        offer_service,
        agreement_service,
        config.clone(),
        ssi_auth_service,
        mates_service,
        validator.clone(),
    )
    .build_router()
    .await
    .expect("Failed to build DSP router");

    // router
    let router_str = format!("{}/negotiation-agent", config.common().get_api_version());
    let management_router = Router::new()
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
        .route_layer(axum::middleware::from_fn_with_state(
            validator,
            common::auth::http::AuthHttpMiddleware::run,
        ));

    Router::new()
        .merge(management_router)
        .nest("/dsp/current/negotiations", dsp_router)
}
