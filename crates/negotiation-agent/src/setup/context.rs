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

//! What every negotiation module uses: config, repos, events, ports and the services built on them.

use std::sync::{Arc, OnceLock};

use common::config::services::ContractsConfig;
use ymir::module_loader::root_context::RootContext;
use ymir::services::token_validator::OauthTokenValidatorTrait;
use events::EventSinkTrait;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::agreement::service::AgreementService;
use crate::services::agreement::AgreementServiceTrait;
use crate::services::negotiation_message::service::NegotiationMessageService;
use crate::services::negotiation_message::NegotiationMessageServiceTrait;
use crate::services::negotiation_process::service::NegotiationProcessService;
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use crate::services::offer::service::OfferService;
use crate::services::offer::OfferServiceTrait;
use crate::setup::ports::NegotiationPorts;

/// The negotiation agent's context; every service, router, flow and loader keeps it whole.
pub struct AppContext {
    pub config: Arc<ContractsConfig>,
    pub repos: Arc<dyn DataFactory>,
    pub events: Arc<dyn EventSinkTrait>,
    pub ports: NegotiationPorts,
    pub oauth_validator: Arc<dyn OauthTokenValidatorTrait>,
    services: OnceLock<NegotiationServices>,
}

/// The agent's own services; they keep the context, so they are attached once it exists.
#[derive(Clone)]
pub struct NegotiationServices {
    pub processes: Arc<dyn NegotiationProcessServiceTrait>,
    pub messages: Arc<dyn NegotiationMessageServiceTrait>,
    pub offers: Arc<dyn OfferServiceTrait>,
    pub agreements: Arc<dyn AgreementServiceTrait>,
}

impl NegotiationServices {
    /// The real services over the context's repos and events.
    pub fn own(ctx: &Arc<AppContext>) -> Self {
        Self {
            processes: Arc::new(NegotiationProcessService::new(ctx.clone())),
            messages: Arc::new(NegotiationMessageService::new(ctx.clone())),
            offers: Arc::new(OfferService::new(ctx.clone())),
            agreements: Arc::new(AgreementService::new(ctx.clone())),
        }
    }
}

impl AppContext {
    /// SeaORM repos on the shared pool, the given events and ports, and the own services.
    pub fn build(
        config: &ContractsConfig,
        root: &RootContext,
        events: Arc<dyn EventSinkTrait>,
        ports: NegotiationPorts,
    ) -> Arc<Self> {
        let ctx = Self::bare(
            Arc::new(config.clone()),
            Arc::new(SeaOrmDataFactory::new(root.db.clone())),
            events,
            ports,
            root.validator.clone(),
        );
        ctx.attach(NegotiationServices::own(&ctx));
        ctx
    }

    /// The context without services; `attach` completes it (tests attach mocks).
    pub fn bare(
        config: Arc<ContractsConfig>,
        repos: Arc<dyn DataFactory>,
        events: Arc<dyn EventSinkTrait>,
        ports: NegotiationPorts,
        oauth_validator: Arc<dyn OauthTokenValidatorTrait>,
    ) -> Arc<Self> {
        Arc::new(Self {
            config,
            repos,
            events,
            ports,
            oauth_validator,
            services: OnceLock::new(),
        })
    }

    /// Attaches the services; only the first call counts.
    pub fn attach(&self, services: NegotiationServices) {
        let _ = self.services.set(services);
    }

    pub fn processes(&self) -> &Arc<dyn NegotiationProcessServiceTrait> {
        &self.services().processes
    }

    pub fn messages(&self) -> &Arc<dyn NegotiationMessageServiceTrait> {
        &self.services().messages
    }

    pub fn offers(&self) -> &Arc<dyn OfferServiceTrait> {
        &self.services().offers
    }

    pub fn agreements(&self) -> &Arc<dyn AgreementServiceTrait> {
        &self.services().agreements
    }

    fn services(&self) -> &NegotiationServices {
        self.services
            .get()
            .expect("services are attached before any module runs")
    }
}
