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

//! Catalog agent as a composable module: gRPC plane, migrations and the tenant listener;
//! the HTTP plane still comes from `create_root_http_router_with_bus` until it migrates.

use crate::grpc::api::catalog_agent::catalog_entity_service_server::CatalogEntityServiceServer;
use crate::grpc::api::catalog_agent::data_service_entity_service_server::DataServiceEntityServiceServer;
use crate::grpc::api::catalog_agent::dataset_entity_service_server::DatasetEntityServiceServer;
use crate::grpc::api::catalog_agent::distribution_entity_service_server::DistributionEntityServiceServer;
use crate::grpc::api::catalog_agent::odrl_policy_entity_service_server::OdrlPolicyEntityServiceServer;
use crate::grpc::api::catalog_agent::policy_template_entity_service_server::PolicyTemplateEntityServiceServer;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::catalogs::CatalogEntityGrpc;
use crate::grpc::data_services::DataServiceEntityGrpc;
use crate::grpc::datasets::DatasetEntityGrpc;
use crate::grpc::distributions::DistributionEntityGrpc;
use crate::grpc::odrl_policies::OdrlPolicyEntityGrpc;
use crate::grpc::policy_templates::PolicyTemplateEntityGrpc;
use crate::services::tenant_provisioning::listener::TenantProvisioningListener;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;
use common::boot::workers::BackgroundWorker;
use common::config::services::CatalogConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;

pub struct CatalogAgentModule {
    ctx: AppContext,
}

impl CatalogAgentModule {
    pub async fn compose(
        config: &CatalogConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Outcome<Self> {
        Ok(Self::new(AppContext::build(config, root, event_bus).await?))
    }

    pub(crate) fn new(ctx: AppContext) -> Self {
        Self { ctx }
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::data::migrations::get_catalog_migrations()
    }
}

impl ServiceModuleTrait for CatalogAgentModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        let ctx = &self.ctx;
        let validator = || ctx.oauth_validator.clone();
        routes
            .add_service(CatalogEntityServiceServer::new(CatalogEntityGrpc::new(
                ctx.catalog_svc.clone(),
                validator(),
            )))
            .add_service(DataServiceEntityServiceServer::new(
                DataServiceEntityGrpc::new(ctx.data_service_svc.clone(), validator()),
            ))
            .add_service(DatasetEntityServiceServer::new(DatasetEntityGrpc::new(
                ctx.dataset_svc.clone(),
                validator(),
            )))
            .add_service(DistributionEntityServiceServer::new(
                DistributionEntityGrpc::new(ctx.distribution_svc.clone(), validator()),
            ))
            .add_service(OdrlPolicyEntityServiceServer::new(
                OdrlPolicyEntityGrpc::new(ctx.odrl_policy_svc.clone(), validator()),
            ))
            .add_service(PolicyTemplateEntityServiceServer::new(
                PolicyTemplateEntityGrpc::new(ctx.policy_template_svc.clone(), validator()),
            ));
    }

    fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
        vec![FILE_DESCRIPTOR_SET]
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
