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

use common::config::types::cache::CacheType;
use common::config::types::traits::CacheConfigTrait;
use common::config::ApplicationConfig;
use common::well_known::WellKnownModule;
use sea_orm_migration::MigrationTrait;
use ymir::boot::seeders::{BootSeeder, RedisCacheFlush};
use ymir::boot::BootstrapServiceTrait;
use ymir::config::traits::CommonConfigTrait;
use ymir::errors::Outcome;
use ymir::module_loader::root_context::RootContext;
use ymir::module_loader::service_composer::ServiceComposer;

use crate::setup::composition::MonolithModule;

/// Every agent in one process, behind one composer and one database.
pub struct CoreBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for CoreBoot {
    type Config = ApplicationConfig;

    fn banner(big_name: &str) -> String {
        common::info_banner::banner(big_name)
    }

    const MIGRATION_TABLE: &'static str = "seaql_ds_agent_migrations";

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        MonolithModule::migrations()
    }

    async fn compose(config: &ApplicationConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let monolith = MonolithModule::compose(config, root).await?;
        let ports = monolith.auth_ports();
        Ok(ServiceComposer::new()
            .register(monolith)
            .register(WellKnownModule::new(
                config.common(),
                Some(ports.mates.clone()),
            )?))
    }

    /// Infrastructure only, straight to the DB; module seeders come from the composed graph.
    /// The Redis flush only runs when the cache is Redis.
    async fn seeders(
        config: &ApplicationConfig,
        _root: &RootContext,
    ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
        let mut seeders: Vec<Box<dyn BootSeeder>> = Vec::new();
        if matches!(
            config.monolith().cache_config().cache_type,
            CacheType::Redis
        ) {
            seeders.push(Box::new(RedisCacheFlush::new(
                config.monolith().get_full_cache_url()?,
            )));
        }
        // seeders.push(Box::new(AdminSeeder::new(root.db.clone(), config.common())));
        Ok(seeders)
    }
}
