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

use crate::setup::context::CoreContext;
use auth::data::migrations::get_auth_migrations;
use axum::Router;
use catalog_agent::get_catalog_migrations;
use catalog_agent::setup::CatalogAgentModule;
use common::boot::workers::BackgroundWorker;
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use common::module_loader::to_be_deprecated::ToBeDeprecatedRouterModule;
use connector::get_connector_migrations;
use dataplane::get_dataplane_migrations;
use events::data::migrations::get_events_migrations;
use keystore::KeystoreModule;
use negotiation_agent::get_negotiation_agent_migrations;
use negotiation_agent::setup::NegotiationAgentModule;
use oauth::get_oauth_migrations;
use oauth::setup::module::OAuthModule;
use sea_orm_migration::MigrationTrait;
use std::sync::Arc;
use tonic::service::RoutesBuilder;
use transfer_agent::setup::TransferAgentModule;
use ymir::errors::Outcome;

pub struct MonolithModule {
    group: ModuleGroup,
}

impl MonolithModule {
    pub async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<Self> {
        let ctx = CoreContext::build(config, root).await?;
        let bus = (*ctx.events_ctx.event_bus).clone();

        let transfer = TransferAgentModule::compose(config.transfer(), root, Some(bus.clone()));
        // Catalog and negotiation contribute their gRPC plane as modules; their HTTP plane
        // still comes through the transitional router wrappers below.
        let catalog_grpc =
            CatalogAgentModule::compose(config.catalog(), root, Some(bus.clone())).await?;
        let negotiation_grpc = NegotiationAgentModule::compose(root, Some(bus.clone()));
        let oauth = OAuthModule::new(config.common().clone().into(), root.db.clone())
            .with_event_bus(Some(bus.clone()));
        let keystore =
            KeystoreModule::build(config.monolith(), Arc::new(config.clone()), root, Some(bus));

        // Preserve the previous `create_core_router` mount layout: each agent
        // merged at the root, keystore nested under `{api}/keystore`.
        let group = ModuleGroup::new("monolith")
            .register(ToBeDeprecatedRouterModule::merged(
                "catalog-agent",
                ctx.catalog_router,
            ))
            .register(catalog_grpc)
            .register(ctx.auth)
            .register(ToBeDeprecatedRouterModule::merged(
                "negotiation-agent",
                ctx.negotiation_router,
            ))
            .register(negotiation_grpc)
            .register(oauth)
            .register(transfer)
            .register(events::setup::composition::EventsModule::new(
                ctx.events_ctx.clone(),
                root.validator.clone(),
            ))
            .register(ctx.gateway)
            .register(keystore);

        Ok(Self { group })
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            get_catalog_migrations(),
            get_connector_migrations(),
            get_negotiation_agent_migrations(),
            get_events_migrations(),
            get_auth_migrations(),
            get_dataplane_migrations(),
            get_oauth_migrations(),
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
        self.group.http()
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        self.group.grpc(routes);
    }

    fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
        self.group.grpc_descriptors()
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        self.group.workers()
    }
}
