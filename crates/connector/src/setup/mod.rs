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

use crate::data::factory_sql::ConnectorRepoForSql;
use crate::data::factory_trait::ConnectorRepoTrait;
use crate::facades::distribution_resolver_facade::data_service_resolver_facade::DistributionFacadeServiceForConnector;
use crate::http::connector_instance::ConnectorInstanceRouter;
use crate::http::connector_template::ConnectorTemplateRouter;
use crate::services::connector_instance::service::ConnectorInstanceService;
use crate::services::connector_instance::ConnectorInstanceServiceTrait;
use crate::services::connector_template::service::ConnectorTemplateService;
use axum::Router;
use common::config::services::CatalogConfig;
use common::config::types::traits::CommonConfigTrait;
use common::http_client::HttpClient;
use common::module_loader::root_context::RootContext;
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

pub struct ConnectorSetup {}
impl ConnectorSetup {
    pub fn new() -> Self {
        ConnectorSetup {}
    }

    pub fn get_connector_repo(&self, root: &RootContext) -> Arc<dyn ConnectorRepoTrait> {
        Arc::new(ConnectorRepoForSql::create_repo(root.db.clone()))
    }

    pub fn get_connector_instance_entity<C: CommonConfigTrait + Send + Sync>(
        &self,
        config: &C,
        root: &RootContext,
        http_client: Arc<HttpClient>,
        event_bus: Option<events::EventBus>,
    ) -> Arc<dyn ConnectorInstanceServiceTrait> {
        let distribution_facade = Arc::new(DistributionFacadeServiceForConnector::new(
            config,
            http_client,
        ));
        let own_url = config.common().get_host(HostType::Http);
        Arc::new(
            ConnectorInstanceService::new(
                self.get_connector_repo(root),
                distribution_facade,
                own_url,
            )
            .with_event_bus(event_bus),
        )
    }

    pub fn build_control_router(
        &self,
        config: &CatalogConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Router {
        let connector_repo = self.get_connector_repo(root);
        let config_arc = Arc::new(config.clone());
        let http_client = Arc::new(HttpClient::new(3, 1));

        let connector_template_service = Arc::new(
            ConnectorTemplateService::new(connector_repo.clone()).with_event_bus(event_bus.clone()),
        );
        let connector_template_router =
            ConnectorTemplateRouter::new(connector_template_service.clone(), config_arc.clone())
                .router();
        let connector_instance_router = ConnectorInstanceRouter::new(
            self.get_connector_instance_entity(config, root, http_client, event_bus),
        )
        .router();

        Router::new()
            .nest("/templates", connector_template_router)
            .nest("/instances", connector_instance_router)
            .route_layer(axum::middleware::from_fn_with_state(
                root.validator.clone(),
                common::auth::http::AuthHttpMiddleware::run,
            ))
    }
}
