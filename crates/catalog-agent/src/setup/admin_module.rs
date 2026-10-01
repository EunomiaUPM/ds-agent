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

//! Management plane of the catalog agent: the `{api}/catalog-agent` HTTP API and its gRPC mirror.

use std::sync::Arc;

use axum::Router;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::service_module::ServiceModuleTrait;
use tonic::service::RoutesBuilder;
use ymir::config::traits::ApiConfigTrait;

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
use crate::http::catalogs::CatalogEntityRouter;
use crate::http::data_services::DataServiceEntityRouter;
use crate::http::dataset_offerings::DatasetOfferingRouter;
use crate::http::datasets::DatasetEntityRouter;
use crate::http::distributions::DistributionEntityRouter;
use crate::http::odrl_policies::OdrlOfferEntityRouter;
use crate::http::peer_catalog::PeerCatalogEntityRouter;
use crate::http::policy_templates::PolicyTemplateEntityRouter;
use crate::http::tenants::TenantRouter;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

pub(crate) struct CatalogAdminModule {
    ctx: Arc<AppContext>,
}

impl CatalogAdminModule {
    pub(crate) fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Mount prefix of this agent's own API, e.g. `/api/v1/catalog-agent`.
    fn base_path(&self) -> String {
        format!(
            "{}/{SERVICE_NAME}",
            self.ctx.config.common().get_api_version()
        )
    }
}

impl ServiceModuleTrait for CatalogAdminModule {
    fn name(&self) -> &'static str {
        "catalog-admin"
    }

    fn http(&self) -> Option<(String, Router)> {
        let ctx = &self.ctx;
        let config = ctx.config.clone();
        let router = Router::new()
            .nest(
                "/catalogs",
                CatalogEntityRouter::new(ctx.catalog_svc.clone(), config.clone()).router(),
            )
            .nest(
                "/data-services",
                DataServiceEntityRouter::new(ctx.data_service_svc.clone(), config.clone()).router(),
            )
            .nest(
                "/datasets",
                DatasetEntityRouter::new(ctx.dataset_svc.clone(), config.clone()).router(),
            )
            .nest(
                "/distributions",
                DistributionEntityRouter::new(ctx.distribution_svc.clone(), config.clone())
                    .router(),
            )
            .nest(
                "/odrl-policies",
                OdrlOfferEntityRouter::new(ctx.odrl_policy_svc.clone(), config.clone()).router(),
            )
            .nest(
                "/policy-templates",
                PolicyTemplateEntityRouter::new(
                    ctx.policy_template_svc.clone(),
                    ctx.policy_instantiation_svc.clone(),
                    config,
                )
                .router(),
            )
            .nest(
                "/peer-catalogs",
                PeerCatalogEntityRouter::new(ctx.peer_catalog_svc.clone()).router(),
            )
            .nest(
                "/dataset-offerings",
                DatasetOfferingRouter::new(ctx.dataset_offering_svc.clone()).router(),
            )
            .nest(
                "/tenants",
                TenantRouter::new(ctx.tenant_provisioning_svc.clone()).router(),
            )
            .route_layer(axum::middleware::from_fn_with_state(
                ctx.oauth_validator.clone(),
                common::auth::http::AuthHttpMiddleware::run,
            ));
        Some((self.base_path(), router))
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
}
