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

//! Wires the negotiation data layer and management services once, for every driving adapter.

use std::sync::Arc;

use crate::data::factory_sql::NegotiationAgentRepoForSql;
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::services::agreement::AgreementServiceTrait;
use crate::services::agreement::service::AgreementService;
use crate::services::negotiation_message::NegotiationMessageServiceTrait;
use crate::services::negotiation_message::service::NegotiationMessageService;
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use crate::services::negotiation_process::service::NegotiationProcessService;
use crate::services::offer::OfferServiceTrait;
use crate::services::offer::service::OfferService;
use common::auth::OauthTokenValidator;
use common::module_loader::root_context::RootContext;

#[derive(Clone)]
pub struct AppContext {
    pub process_svc: Arc<dyn NegotiationProcessServiceTrait>,
    pub message_svc: Arc<dyn NegotiationMessageServiceTrait>,
    pub offer_svc: Arc<dyn OfferServiceTrait>,
    pub agreement_svc: Arc<dyn AgreementServiceTrait>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
}

impl AppContext {
    pub fn build(root: &RootContext, event_bus: Option<events::EventBus>) -> Self {
        let repo = Arc::new(NegotiationAgentRepoForSql::create_repo(root.db.clone()));

        // Domain services
        let process_svc = Arc::new(
            NegotiationProcessService::new(
                repo.get_negotiation_process_repo(),
                repo.get_negotiation_process_identifiers_repo(),
                repo.get_negotiation_message_repo(),
                repo.get_offer_repo(),
                repo.get_agreement_repo(),
            )
            .with_event_bus(event_bus.clone()),
        );
        let message_svc = Arc::new(
            NegotiationMessageService::new(
                repo.get_negotiation_message_repo(),
                repo.get_offer_repo(),
                repo.get_agreement_repo(),
            )
            .with_event_bus(event_bus.clone()),
        );
        let offer_svc =
            Arc::new(OfferService::new(repo.get_offer_repo()).with_event_bus(event_bus.clone()));
        let agreement_svc =
            Arc::new(AgreementService::new(repo.get_agreement_repo()).with_event_bus(event_bus));

        Self {
            process_svc,
            message_svc,
            offer_svc,
            agreement_svc,
            oauth_validator: root.validator.clone(),
        }
    }
}
