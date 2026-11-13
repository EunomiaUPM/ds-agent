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

use std::sync::Arc;

use auth::setup::{AuthModule, SelfParticipantOnboarder};
use axum::Router;
use bff::BffModule;
use catalog_agent::setup::{CatalogAgentModule, CatalogPorts};
use common::config::ApplicationConfig;
use common::facades::AuthPorts;
use common::public_refs::PublicResolverTrait;
use events::setup::EventsModule;
use keystore::KeystoreModule;
use negotiation_agent::setup::{NegotiationAgentModule, NegotiationPorts};
use ymir::boot::seeders::BootSeeder;
use ymir::boot::workers::BackgroundWorker;
use ymir::config::traits::{ApiConfigTrait, CommonConfigTrait};
use ymir::module_loader::module_group::ModuleGroup;
use ymir::module_loader::root_context::RootContext;
use ymir::module_loader::service_module::ServiceModuleTrait;
// The built-in OAuth crate is disabled (identity from Keycloak or the static user).
// use oauth::setup::OAuthModule;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use transfer_agent::setup::{TransferAgentModule, TransferPorts};
use ymir::errors::Outcome;

use crate::http::LandingRouter;

/// Every agent module of the process, plus the auth ports and the own-wallet onboarding.
pub struct MonolithModule {
    modules: ModuleGroup,
    api_base: String,
    auth_ports: AuthPorts,
    public_resolvers: Vec<Arc<dyn PublicResolverTrait>>,
    self_participant: SelfParticipantOnboarder,
}

impl MonolithModule {
    /// Providers are built before their consumers so every facade resolves in-process;
    /// registration order is kept as before.
    pub async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<Self> {
        let events = EventsModule::try_compose(root)?;
        let sink = events.event_sink();
        let auth = AuthModule::compose(config.ssi_auth()?, root, sink.clone()).await?;
        let auth_ports = auth.local_ports();
        let self_participant = auth.self_participant_onboarder();
        let transfer_guard = Arc::new(connector::LateBoundTransferGuard::new());
        let catalog_ports =
            CatalogPorts::local(auth_ports.clone()).with_transfer_guard(transfer_guard.clone());
        let catalog =
            CatalogAgentModule::compose(config.catalog()?, root, sink.clone(), &catalog_ports)
                .await?;
        let negotiation_ports =
            NegotiationPorts::local(auth_ports.clone(), catalog.catalog_facade());
        let negotiation = NegotiationAgentModule::compose(
            config.contracts()?,
            root,
            sink.clone(),
            &negotiation_ports,
        )
        .await?;
        // Agreements first: their URNs are typed, the catalog's may be any.
        let public_resolvers = vec![negotiation.public_resolver(), catalog.public_resolver()];
        let transfer_ports = TransferPorts::local(
            config.transfer()?,
            root,
            sink.clone(),
            auth_ports.clone(),
            negotiation.negotiation_facade(),
            catalog.catalog_facade(),
            catalog.local_connector_instances(),
        )
        .await?;
        if let Some(dp) = transfer_ports.dataplane_module() {
            transfer_guard.bind(dp.active_transfer_guard());
        }
        let modules = ModuleGroup::new("monolith")
            .register(catalog)
            .register(auth)
            .register(negotiation)
            // .register(OAuthModule::compose(config.common(), root, sink.clone()))
            .register(TransferAgentModule::compose(
                config.transfer()?,
                root,
                sink.clone(),
                &transfer_ports,
            ))
            .register(events)
            .register(BffModule::compose(config.gateway()?, root).without_root())
            .register(KeystoreModule::compose(config, root, sink));
        let api_base = config.common().get_api_version();
        Ok(Self {
            modules,
            api_base,
            auth_ports,
            public_resolvers,
            self_participant,
        })
    }

    /// Auth ports resolved in-process, reused by the process-wide surfaces.
    pub fn auth_ports(&self) -> AuthPorts {
        self.auth_ports.clone()
    }

    /// The public references of every agent in the process.
    pub fn public_resolvers(&self) -> Vec<Arc<dyn PublicResolverTrait>> {
        self.public_resolvers.clone()
    }

    /// Static aggregation in cross-crate FK order; `MigratorTrait` cannot drive `compose`.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            CatalogAgentModule::migrations(),
            NegotiationAgentModule::migrations(),
            EventsModule::migrations(),
            AuthModule::migrations(),
            // OAuthModule::migrations(),
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
        let landing = LandingRouter::for_api_base(&self.api_base).router();
        let router = match self.modules.http() {
            Some((prefix, router)) if prefix.is_empty() => router.merge(landing),
            Some((prefix, router)) => Router::new().nest(&prefix, router).merge(landing),
            None => landing,
        };
        Some((String::new(), router))
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

    /// The own-wallet onboarding (monolith only) first, then every module's seeders.
    fn seeders(&self) -> Vec<Box<dyn BootSeeder>> {
        let mut seeders: Vec<Box<dyn BootSeeder>> = vec![Box::new(self.self_participant.clone())];
        seeders.extend(self.modules.seeders());
        seeders
    }
}
