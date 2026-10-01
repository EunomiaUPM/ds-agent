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

use common::auth::OauthTokenValidator;
use common::boot::seeders::{BootSeeder, RedisCacheFlush};
use common::boot::BootstrapServiceTrait;
use common::config::services::CommonConfig;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait};
use common::config::ApplicationConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use oauth::setup::AdminSeeder;
use oauth::setup::OAuthModule;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use ymir::errors::Outcome;

use crate::setup::composition::MonolithModule;

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
        OAuthModule::validator(common, db)
    }

    async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let monolith = MonolithModule::compose(config, root).await?;
        let ports = monolith.auth_ports();
        Ok(ServiceComposer::new()
            .register(monolith)
            .with_auth_ports(ports))
    }

    /// Infrastructure only, straight to the DB; module seeders come from the composed graph.
    async fn seeders(
        config: &ApplicationConfig,
        root: &RootContext,
    ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
        Ok(vec![
            Box::new(RedisCacheFlush::new(config.monolith().get_full_cache_url())),
            Box::new(AdminSeeder::new(root.db.clone(), config.common())),
        ])
    }
}
