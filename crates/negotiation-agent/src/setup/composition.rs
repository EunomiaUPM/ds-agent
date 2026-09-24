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
use crate::setup::admin_module::NegotiationAdminModule;
use crate::setup::context::AppContext;
use axum::Router;
use common::config::services::ContractsConfig;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;

pub struct NegotiationAgentModule {
    modules: ModuleGroup,
}

impl NegotiationAgentModule {
    pub async fn compose(
        config: &ContractsConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Outcome<Self> {
        let ctx = Arc::new(AppContext::build(config, root, event_bus));
        let modules = ModuleGroup::new(SERVICE_NAME)
            .register(DspModule::build(ctx.clone()).await?)
            .register(NegotiationAdminModule::new(ctx));
        Ok(Self { modules })
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::data::migrations::get_negotiation_agent_migrations()
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
}
