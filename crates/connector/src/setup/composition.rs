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

//! Connector as a composable module, mounted by the catalog agent under `{api}/connector`.

use axum::Router;
use common::config::services::CatalogConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::ApiConfigTrait;

use crate::http::connector_instance::ConnectorInstanceRouter;
use crate::http::connector_template::ConnectorTemplateRouter;
use crate::setup::context::AppContext;

pub struct ConnectorModule {
    prefix: String,
    ctx: AppContext,
}

impl ConnectorModule {
    pub fn compose(
        config: &CatalogConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        Self {
            prefix: format!("{}/connector", config.common().get_api_version()),
            ctx: AppContext::build(config, root, event_bus),
        }
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::get_connector_migrations()
    }
}

impl ServiceModuleTrait for ConnectorModule {
    fn name(&self) -> &'static str {
        "connector"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let ctx = &self.ctx;
        let router = Router::new()
            .nest(
                "/templates",
                ConnectorTemplateRouter::new(ctx.template_svc.clone(), ctx.config.clone()).router(),
            )
            .nest(
                "/instances",
                ConnectorInstanceRouter::new(ctx.instance_svc.clone()).router(),
            )
            .route_layer(axum::middleware::from_fn_with_state(
                ctx.oauth_validator.clone(),
                common::auth::http::AuthHttpMiddleware::run,
            ));
        Some((self.prefix.clone(), router))
    }
}
