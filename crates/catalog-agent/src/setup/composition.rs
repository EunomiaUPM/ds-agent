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

//! Catalog agent as a composable module: management API, DSP catalog, the profile modules, the
//! connector and the seeders of the connector's own records, all over one `AppContext`.

use std::sync::Arc;

use axum::Router;
use common::boot::seeders::BootSeeder;
use common::boot::workers::BackgroundWorker;
use common::config::services::traits::CatalogConfigTrait;
use common::config::services::CatalogConfig;
use common::facades::catalog::CatalogFacadeTrait;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use connector::{ConnectorInstanceFacadeTrait, ConnectorModule, ConnectorPorts};
use events::NoopEventSink;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;

use crate::cache::CatalogCacheFactory;
use crate::data::sea_orm::migrations::CatalogMigrations;
use crate::facades::CatalogLocalFacade;
use crate::profiles::dcat3::Dcat3ProfileModule;
use crate::profiles::dcat_ap::DcatApProfileModule;
use crate::profiles::dsp::DspProfileModule;
use crate::profiles::ProfileModules;
use crate::protocols::dsp::setup::DspModule;
use crate::setup::admin_module::CatalogAdminModule;
use crate::setup::context::AppContext;
use crate::setup::ports::CatalogPorts;
use crate::setup::seeders::{MainCatalogProvisioner, PolicyTemplateLoader};
use crate::SERVICE_NAME;

/// Catalog agent as a module: management API, DSP catalog, profiles and the connector.
pub struct CatalogAgentModule {
    ctx: Arc<AppContext>,
    modules: ModuleGroup,
    catalog_facade: Arc<dyn CatalogFacadeTrait>,
    connector_instances: Arc<dyn ConnectorInstanceFacadeTrait>,
}

impl CatalogAgentModule {
    /// Builds the agent and its connector on the shared root context and the given ports.
    pub async fn compose(
        config: &CatalogConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
        ports: &CatalogPorts,
    ) -> Outcome<Self> {
        let cache = CatalogCacheFactory::from_config(config).await?;
        let events = NoopEventSink::or_bus(event_bus.clone());
        let ctx = AppContext::build(config, root, cache, events, ports.clone());
        let catalog_facade: Arc<dyn CatalogFacadeTrait> = Arc::new(CatalogLocalFacade::new(ctx.clone()));
        let connector = ConnectorModule::compose(
            config,
            root,
            event_bus,
            &ConnectorPorts::local(catalog_facade.clone()),
        );
        let connector_instances = connector.local_connector_instances();
        // A new profile is one more module here: its routes mount and the doctor sees it.
        let profiles = ProfileModules::new(ctx.clone())
            .register(DspProfileModule::default())
            .register(Dcat3ProfileModule::new(ctx.clone()))
            .register(DcatApProfileModule::new(ctx.clone()));
        let modules = ModuleGroup::new(SERVICE_NAME)
            .register(DspModule::new(ctx.clone()))
            .register(CatalogAdminModule::new(ctx.clone()))
            .register(profiles)
            .register(connector);
        Ok(Self {
            ctx,
            modules,
            catalog_facade,
            connector_instances,
        })
    }

    /// Connector instances served in-process, for agents sharing this process.
    pub fn local_connector_instances(&self) -> Arc<dyn ConnectorInstanceFacadeTrait> {
        self.connector_instances.clone()
    }

    /// The catalog's published contract, for agents that read it in-process.
    pub fn catalog_facade(&self) -> Arc<dyn CatalogFacadeTrait> {
        self.catalog_facade.clone()
    }

    /// Catalog then connector, the FK order the monolith relies on.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [CatalogMigrations::all(), ConnectorModule::migrations()]
            .into_iter()
            .flatten()
            .collect()
    }
}

impl ServiceModuleTrait for CatalogAgentModule {
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

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        self.modules.workers()
    }

    /// The connector's main catalog and the policy template library, on the services
    /// in-process, as the root: both are the connector's and public.
    fn seeders(&self) -> Vec<Box<dyn BootSeeder>> {
        let mut seeders: Vec<Box<dyn BootSeeder>> = vec![
            Box::new(MainCatalogProvisioner::new(self.ctx.clone())),
            Box::new(PolicyTemplateLoader::new(
                self.ctx.clone(),
                self.ctx.config.get_policy_templates_folder().to_string(),
            )),
        ];
        seeders.extend(self.modules.seeders());
        seeders
    }
}
