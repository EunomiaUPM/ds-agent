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

//! Every agent as one module group, sharing the root context and a single event bus.

use auth::setup::AuthModule;
use axum::Router;
use bff::BffModule;
use catalog_agent::setup::CatalogAgentModule;
use common::boot::workers::BackgroundWorker;
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use dataplane::get_dataplane_migrations;
use events::setup::EventsModule;
use keystore::KeystoreModule;
use negotiation_agent::setup::NegotiationAgentModule;
use oauth::setup::OAuthModule;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use transfer_agent::setup::TransferAgentModule;
use ymir::errors::Outcome;

pub struct MonolithModule {
    modules: ModuleGroup,
}

impl MonolithModule {
    pub async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<Self> {
        let events = EventsModule::compose(root);
        let bus = Some(events.event_bus());
        let modules = ModuleGroup::new("monolith")
            .register(CatalogAgentModule::compose(config.catalog(), root, bus.clone()).await?)
            .register(AuthModule::compose(config.ssi_auth(), root).await?)
            .register(NegotiationAgentModule::compose(config.contracts(), root, bus.clone()).await?)
            .register(OAuthModule::compose(config.common(), root, bus.clone()))
            .register(TransferAgentModule::compose(
                config.transfer(),
                root,
                bus.clone(),
            ))
            .register(events)
            .register(BffModule::compose(config.gateway(), root))
            .register(KeystoreModule::compose(config, root, bus));
        Ok(Self { modules })
    }

    /// Static aggregation in cross-crate FK order; `MigratorTrait` cannot drive `compose`.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            CatalogAgentModule::migrations(),
            NegotiationAgentModule::migrations(),
            EventsModule::migrations(),
            AuthModule::migrations(),
            get_dataplane_migrations(),
            OAuthModule::migrations(),
            TransferAgentModule::migrations(),
            KeystoreModule::migrations(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl ServiceModuleTrait for MonolithModule {
    fn name(&self) -> &'static str {
        "monolith"
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

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        self.modules.workers()
    }
}
