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

use std::sync::Arc;

use common::auth::{OauthTokenValidator, ServiceHttpClient};
use common::boot::seeders::BootSeeder;
use common::boot::BootstrapServiceTrait;
use common::config::services::traits::CatalogConfigTrait;
use common::config::services::{CatalogConfig, CommonConfig};
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use common::module_loader::to_be_deprecated::ToBeDeprecatedRouterModule;
use connector::get_connector_migrations;
use oauth::setup::composition::OAuthSetup;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use crate::setup::http_router::create_root_http_router;
use crate::setup::seeders::{AdminTenantProvisioner, PolicyTemplateLoader};
use crate::setup::CatalogAgentModule;
use crate::SERVICE_NAME;

pub struct CatalogAgentBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for CatalogAgentBoot {
    type Config = CatalogConfig;

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [CatalogAgentModule::migrations(), get_connector_migrations()]
            .into_iter()
            .flatten()
            .collect()
    }

    fn validator(common: &CommonConfig, db: DatabaseConnection) -> Arc<dyn OauthTokenValidator> {
        OAuthSetup::validator(common, db)
    }

    async fn compose(config: &CatalogConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let http = create_root_http_router(config, root, None).await?;
        Ok(ServiceComposer::new()
            .register(ToBeDeprecatedRouterModule::merged(SERVICE_NAME, http))
            .register(CatalogAgentModule::compose(config, root, None).await?))
    }

    async fn seeders(
        config: &CatalogConfig,
        _root: &RootContext,
    ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
        let common = config.common();
        let client = Arc::new(ServiceHttpClient::from_common(common, 30));
        let api_url = format!(
            "{}{}/{SERVICE_NAME}",
            common.get_host(HostType::Http),
            common.get_api_version()
        );
        let tenant = config.admin_seed().tenant_id.clone();
        let folder = config.get_policy_templates_folder().to_string();
        Ok(vec![
            Box::new(AdminTenantProvisioner::new(
                client.clone(),
                api_url.clone(),
                tenant.clone(),
            )),
            Box::new(PolicyTemplateLoader::new(client, api_url, tenant, folder)),
        ])
    }
}
