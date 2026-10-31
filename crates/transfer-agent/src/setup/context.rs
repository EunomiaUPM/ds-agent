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

//! What every transfer module uses: config, repos, events, ports and the services built on them.

use std::sync::{Arc, OnceLock};

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_process::service::TransferProcessService;
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::setup::ports::TransferPorts;
use ymir::services::token_validator::OauthTokenValidatorTrait;
use events::EventSinkTrait;
use common::config::services::TransferConfig;
use ymir::module_loader::root_context::RootContext;

/// The transfer agent's context; every service, router, flow and loader keeps it whole.
pub struct AppContext {
    pub config: Arc<TransferConfig>,
    pub repos: Arc<dyn DataFactory>,
    pub events: Arc<dyn EventSinkTrait>,
    pub ports: TransferPorts,
    pub oauth_validator: Arc<dyn OauthTokenValidatorTrait>,
    services: OnceLock<TransferServices>,
}

/// The agent's own services; they keep the context, so they are attached once it exists.
#[derive(Clone)]
pub struct TransferServices {
    pub processes: Arc<dyn TransferProcessServiceTrait>,
    pub messages: Arc<dyn TransferMessageServiceTrait>,
}

impl TransferServices {
    /// The real services over the context's repos and events.
    pub fn own(ctx: &Arc<AppContext>) -> Self {
        Self {
            processes: Arc::new(TransferProcessService::new(ctx.clone())),
            messages: Arc::new(TransferMessageService::new(ctx.clone())),
        }
    }
}

impl AppContext {
    /// SeaORM repos on the shared pool, the given events and ports, and the own services.
    pub fn build(
        config: &TransferConfig,
        root: &RootContext,
        events: Arc<dyn EventSinkTrait>,
        ports: TransferPorts,
    ) -> Arc<Self> {
        let ctx = Self::bare(
            Arc::new(config.clone()),
            Arc::new(SeaOrmDataFactory::new(root.db.clone())),
            events,
            ports,
            root.validator.clone(),
        );
        ctx.attach(TransferServices::own(&ctx));
        ctx
    }

    /// The context without services; `attach` completes it (tests attach mocks).
    pub fn bare(
        config: Arc<TransferConfig>,
        repos: Arc<dyn DataFactory>,
        events: Arc<dyn EventSinkTrait>,
        ports: TransferPorts,
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
    pub fn attach(&self, services: TransferServices) {
        let _ = self.services.set(services);
    }

    pub fn processes(&self) -> &Arc<dyn TransferProcessServiceTrait> {
        &self.services().processes
    }

    pub fn messages(&self) -> &Arc<dyn TransferMessageServiceTrait> {
        &self.services().messages
    }

    fn services(&self) -> &TransferServices {
        self.services
            .get()
            .expect("services are attached before any module runs")
    }
}
