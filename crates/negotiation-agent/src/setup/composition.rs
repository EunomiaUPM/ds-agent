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

//! Negotiation agent as a composable module: management API and DSP negotiations over one
//! `AppContext`.

use std::sync::Arc;

use crate::SERVICE_NAME;
use crate::protocols::dsp::setup::DspModule;
use crate::facades::NegotiationLocalFacade;
use crate::setup::admin_module::NegotiationAdminModule;
use crate::setup::context::AppContext;
use crate::setup::ports::NegotiationPorts;
use axum::Router;
use common::config::services::ContractsConfig;
use common::facades::negotiation::NegotiationFacadeTrait;
use ymir::module_loader::module_group::ModuleGroup;
use ymir::module_loader::root_context::RootContext;
use ymir::module_loader::service_module::ServiceModuleTrait;
use events::NoopEventSink;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;

/// Negotiation agent as a module: management API and DSP negotiation.
pub struct NegotiationAgentModule {
    ctx: Arc<AppContext>,
    modules: ModuleGroup,
}

impl NegotiationAgentModule {
    /// Builds the agent on the shared root context and the given ports.
    pub async fn compose(
        config: &ContractsConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
        ports: &NegotiationPorts,
    ) -> Outcome<Self> {
        let events = NoopEventSink::or_bus(event_bus);
        let ctx = AppContext::build(config, root, events, ports.clone());
        let modules = ModuleGroup::new(SERVICE_NAME)
            .register(DspModule::new(ctx.clone()))
            .register(NegotiationAdminModule::new(ctx.clone()));
        Ok(Self { ctx, modules })
    }

    /// The negotiation agent's published contract, for agents that read it in-process.
    pub fn negotiation_facade(&self) -> Arc<dyn NegotiationFacadeTrait> {
        Arc::new(NegotiationLocalFacade::new(self.ctx.agreements().clone()))
    }

    /// Static, so the hosting agent's migrator can list them.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::data::sea_orm::migrations::get_negotiation_agent_migrations()
    }
}

impl ServiceModuleTrait for NegotiationAgentModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        self.modules.http()
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        self.modules.grpc(routes);
    }

    fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
        self.modules.grpc_descriptors()
    }

    fn workers(&self) -> Vec<Box<dyn ymir::boot::workers::BackgroundWorker>> {
        self.modules.workers()
    }
}
