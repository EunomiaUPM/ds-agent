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

//! Catalog agent as a composable module: management API, DSP catalog, connector and the
//! tenant listener, all over one `AppContext`.

use std::sync::Arc;

use crate::protocols::dsp::setup::DspModule;
use crate::services::tenant_provisioning::listener::TenantProvisioningListener;
use crate::setup::admin_module::CatalogAdminModule;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;
use axum::Router;
use common::boot::workers::BackgroundWorker;
use common::config::services::CatalogConfig;
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use connector::ConnectorModule;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;

pub struct CatalogAgentModule {
    ctx: Arc<AppContext>,
    modules: ModuleGroup,
}

impl CatalogAgentModule {
    pub async fn compose(
        config: &CatalogConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Outcome<Self> {
        let ctx = Arc::new(AppContext::build(config, root, event_bus.clone()).await?);
        let modules = ModuleGroup::new(SERVICE_NAME)
            .register(DspModule::build(ctx.clone()).await?)
            .register(CatalogAdminModule::new(ctx.clone()))
            .register(ConnectorModule::compose(config, root, event_bus));
        Ok(Self { ctx, modules })
    }

    /// Catalog then connector, the FK order the monolith relies on.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [
            crate::data::migrations::get_catalog_migrations(),
            ConnectorModule::migrations(),
        ]
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

    /// Tenants born elsewhere are only heard through the shared bus.
    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        let Some(bus) = self.ctx.event_bus.clone() else {
            return vec![];
        };
        let listener =
            TenantProvisioningListener::new(bus, self.ctx.tenant_provisioning_svc.clone());
        vec![Box::new(listener)]
    }
}
