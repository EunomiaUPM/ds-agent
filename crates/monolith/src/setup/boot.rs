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

use catalog_agent::setup::{AdminTenantProvisioner, PolicyTemplateLoader};
use common::auth::{OauthTokenValidator, ServiceHttpClient};
use common::boot::seeders::{BootSeeder, RedisCacheFlush};
use common::boot::BootstrapServiceTrait;
use common::config::services::traits::CatalogConfigTrait;
use common::config::services::CommonConfig;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait};
use common::config::ApplicationConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use oauth::setup::composition::OAuthSetup;
use oauth::setup::seeder::AdminSeeder;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use crate::setup::composition::MonolithModule;
use crate::setup::seeders::SelfParticipantOnboarder;

/// Every agent in one process, behind one composer and one database.
pub struct CoreBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for CoreBoot {
    type Config = ApplicationConfig;

    const MIGRATION_TABLE: &'static str = "seaql_ds_agent_migrations";

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        MonolithModule::migrations()
    }

    fn validator(common: &CommonConfig, db: DatabaseConnection) -> Arc<dyn OauthTokenValidator> {
        OAuthSetup::validator(common, db)
    }

    async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        Ok(ServiceComposer::new().register(MonolithModule::compose(config, root).await?))
    }

    async fn seeders(
        config: &ApplicationConfig,
        root: &RootContext,
    ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
        let common = config.common();
        let client = Arc::new(ServiceHttpClient::from_common(common, 30));
        let catalog_api = format!(
            "{}{}/{}",
            common.get_host(HostType::Http),
            common.get_api_version(),
            catalog_agent::SERVICE_NAME
        );
        let tenant = common.admin_seed.tenant_id.clone();
        let templates = config.catalog().get_policy_templates_folder().to_string();
        Ok(vec![
            Box::new(RedisCacheFlush::new(config.monolith().get_full_cache_url())),
            Box::new(AdminSeeder::new(root.db.clone(), common)),
            Box::new(SelfParticipantOnboarder::new(common.clone())),
            Box::new(AdminTenantProvisioner::new(
                client.clone(),
                catalog_api.clone(),
                tenant.clone(),
            )),
            Box::new(PolicyTemplateLoader::new(
                client,
                catalog_api,
                tenant,
                templates,
            )),
        ])
    }
}
