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

use axum::Router;
use common::config::services::TransferConfig;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use dataplane::setup::DataplaneModule;
use sea_orm_migration::MigrationTrait;
use std::sync::Arc;
use tonic::service::RoutesBuilder;

use crate::SERVICE_NAME;
use crate::protocols::dsp::setup::DspModule;
use crate::setup::admin_module::TransferAdminModule;
use crate::setup::context::AppContext;
use crate::setup::ports::TransferPorts;

pub struct TransferAgentModule {
    modules: ModuleGroup,
}

impl TransferAgentModule {
    pub fn compose(
        config: &TransferConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
        ports: &TransferPorts,
    ) -> Self {
        let ctx = Arc::new(AppContext::build(config, root, event_bus, ports));
        let modules = ModuleGroup::new(SERVICE_NAME)
            .register(DspModule::new(ctx.clone()))
            .register(TransferAdminModule::new(ctx))
            .register(ports.dataplane.module.clone());
        Self { modules }
    }

    /// Transfer then its dataplane.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            crate::data::sea_orm::migrations::get_migrations(),
            DataplaneModule::migrations(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl ServiceModuleTrait for TransferAgentModule {
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
