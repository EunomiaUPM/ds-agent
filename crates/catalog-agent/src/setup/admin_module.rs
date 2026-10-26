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
use common::routes::catalog as routes;
use tonic::service::RoutesBuilder;
use ymir::config::traits::ApiConfigTrait;

use crate::entities::resource_kind::ResourceKind;
use crate::grpc::api::catalog_agent::catalog_resource_service_server::CatalogResourceServiceServer;
use crate::grpc::api::catalog_agent::odrl_offer_service_server::OdrlOfferServiceServer;
use crate::grpc::api::catalog_agent::policy_template_service_server::PolicyTemplateServiceServer;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::odrl_offers::OdrlOfferGrpc;
use crate::grpc::policy_templates::PolicyTemplateGrpc;
use crate::grpc::resources::CatalogResourceGrpc;
use crate::http::dataset_offerings::DatasetOfferingRouter;
use crate::http::odrl_policies::OdrlOfferEntityRouter;
use crate::http::peer_catalogs::PeerCatalogRouter;
use crate::http::policy_templates::PolicyTemplateEntityRouter;
use crate::http::resources::ResourceRouter;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

pub struct CatalogAdminModule {
    ctx: Arc<AppContext>,
}

impl CatalogAdminModule {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Mount prefix of this agent's own API, e.g. `/api/v1/catalog-agent`.
    fn base_path(&self) -> String {
        format!("{}/{SERVICE_NAME}", self.ctx.config.common().get_api_version())
    }

    /// The prefix and router of each kind of resource.
    fn resources(&self) -> Vec<(&'static str, Router)> {
        [
            (routes::catalogs::PREFIX, ResourceKind::Catalog),
            (routes::datasets::PREFIX, ResourceKind::Dataset),
            (routes::dataset_series::PREFIX, ResourceKind::DatasetSeries),
            (routes::distributions::PREFIX, ResourceKind::Distribution),
            (routes::data_services::PREFIX, ResourceKind::DataService),
            (routes::agents::PREFIX, ResourceKind::Agent),
            (routes::catalog_records::PREFIX, ResourceKind::CatalogRecord),
        ]
        .into_iter()
        .map(|(prefix, kind)| (prefix, ResourceRouter::new(self.ctx.clone(), kind).router()))
        .collect()
    }
}

impl ServiceModuleTrait for CatalogAdminModule {
    fn name(&self) -> &'static str {
        "catalog-admin"
    }

    fn http(&self) -> Option<(String, Router)> {
        let ctx = &self.ctx;
        let mut router = Router::new();
        for (prefix, resources) in self.resources() {
            router = router.nest(prefix, resources);
        }
        let router = router
            .nest(routes::odrl_policies::PREFIX, OdrlOfferEntityRouter::new(ctx.clone()).router())
            .nest(routes::policy_templates::PREFIX, PolicyTemplateEntityRouter::new(ctx.clone()).router())
            .nest(routes::peer_catalogs::PREFIX, PeerCatalogRouter::new(ctx.clone()).router())
            .nest(routes::dataset_offerings::PREFIX, DatasetOfferingRouter::new(ctx.clone()).router())
            .route_layer(axum::middleware::from_fn_with_state(
                ctx.oauth_validator.clone(),
                ymir::http::OauthHttpMiddleware::run,
            ));
        Some((self.base_path(), router))
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        routes
            .add_service(CatalogResourceServiceServer::new(CatalogResourceGrpc::new(self.ctx.clone())))
            .add_service(OdrlOfferServiceServer::new(OdrlOfferGrpc::new(self.ctx.clone())))
            .add_service(PolicyTemplateServiceServer::new(PolicyTemplateGrpc::new(self.ctx.clone())));
    }

    fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
        vec![FILE_DESCRIPTOR_SET]
    }
}
