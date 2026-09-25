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

use auth::setup::{AuthModule, SelfParticipantOnboarder};
use axum::Router;
use bff::BffModule;
use catalog_agent::setup::{CatalogAgentModule, CatalogPorts};
use common::boot::seeders::BootSeeder;
use common::boot::workers::BackgroundWorker;
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use common::facades::AuthPorts;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use events::setup::EventsModule;
use keystore::KeystoreModule;
use negotiation_agent::setup::{NegotiationAgentModule, NegotiationPorts};
use oauth::setup::OAuthModule;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use transfer_agent::setup::{TransferAgentModule, TransferPorts};
use ymir::errors::Outcome;

pub struct MonolithModule {
    modules: ModuleGroup,
    auth_ports: AuthPorts,
    self_participant: SelfParticipantOnboarder,
}

impl MonolithModule {
    /// Providers are built before their consumers so every facade resolves in-process;
    /// registration order is kept as before.
    pub async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<Self> {
        let events = EventsModule::compose(root);
        let bus = Some(events.event_bus());
        let auth = AuthModule::compose(config.ssi_auth(), root).await?;
        let auth_ports = auth.local_ports();
        let self_participant = auth.self_participant_onboarder();
        let catalog_ports = CatalogPorts::local(auth_ports.clone());
        let catalog =
            CatalogAgentModule::compose(config.catalog(), root, bus.clone(), &catalog_ports)
                .await?;
        let negotiation_ports =
            NegotiationPorts::local(auth_ports.clone(), catalog.odrl_policy_service());
        let negotiation = NegotiationAgentModule::compose(
            config.contracts(),
            root,
            bus.clone(),
            &negotiation_ports,
        )
        .await?;
        let transfer_ports = TransferPorts::local(
            config.transfer(),
            root,
            auth_ports.clone(),
            negotiation.agreement_service(),
            catalog.dataset_service(),
            catalog.distribution_service(),
            catalog.local_connector_instances(),
            config.common().admin_seed.tenant_id.clone(),
        )
        .await?;
        let modules = ModuleGroup::new("monolith")
            .register(catalog)
            .register(auth)
            .register(negotiation)
            .register(OAuthModule::compose(config.common(), root, bus.clone()))
            .register(TransferAgentModule::compose(
                config.transfer(),
                root,
                bus.clone(),
                &transfer_ports,
            ))
            .register(events)
            .register(BffModule::compose(config.gateway(), root))
            .register(KeystoreModule::compose(config, root, bus));
        Ok(Self {
            modules,
            auth_ports,
            self_participant,
        })
    }

    /// Auth ports resolved in-process, reused by the process-wide surfaces.
    pub fn auth_ports(&self) -> AuthPorts {
        self.auth_ports.clone()
    }

    /// Static aggregation in cross-crate FK order; `MigratorTrait` cannot drive `compose`.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            CatalogAgentModule::migrations(),
            NegotiationAgentModule::migrations(),
            EventsModule::migrations(),
            AuthModule::migrations(),
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

    /// The own-wallet onboarding (monolith only) first, then every module's seeders.
    fn seeders(&self) -> Vec<Box<dyn BootSeeder>> {
        let mut seeders: Vec<Box<dyn BootSeeder>> = vec![Box::new(self.self_participant.clone())];
        seeders.extend(self.modules.seeders());
        seeders
    }
}
