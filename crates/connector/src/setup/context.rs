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

//! Wires the connector repository and its template/instance services once.

use std::sync::Arc;

use crate::data::factory_sql::ConnectorRepoForSql;
use crate::facades::distribution_resolver_facade::data_service_resolver_facade::DistributionFacadeServiceForConnector;
use crate::services::connector_instance::service::ConnectorInstanceService;
use crate::services::connector_instance::ConnectorInstanceServiceTrait;
use crate::services::connector_template::service::ConnectorTemplateService;
use crate::services::connector_template::ConnectorTemplateServiceTrait;
use common::auth::OauthTokenValidator;
use common::config::services::CatalogConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

#[derive(Clone)]
pub(crate) struct AppContext {
    pub config: Arc<CatalogConfig>,
    pub template_svc: Arc<dyn ConnectorTemplateServiceTrait>,
    pub instance_svc: Arc<dyn ConnectorInstanceServiceTrait>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
}

impl AppContext {
    pub fn build(
        config: &CatalogConfig,
        root: &RootContext,
        event_bus: Option<events::EventBus>,
    ) -> Self {
        let repo = Arc::new(ConnectorRepoForSql::create_repo(root.db.clone()));
        let distribution_facade = Arc::new(DistributionFacadeServiceForConnector::new(
            config,
            root.service_client.clone(),
        ));
        let instance_svc = Arc::new(
            ConnectorInstanceService::new(
                repo.clone(),
                distribution_facade,
                config.common().get_host(HostType::Http),
            )
            .with_event_bus(event_bus.clone()),
        );
        let template_svc = Arc::new(ConnectorTemplateService::new(repo).with_event_bus(event_bus));
        Self {
            config: Arc::new(config.clone()),
            template_svc,
            instance_svc,
            oauth_validator: root.validator.clone(),
        }
    }
}
