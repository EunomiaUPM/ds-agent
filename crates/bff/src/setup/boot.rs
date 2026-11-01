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

use common::config::services::traits::GatewayConfigTrait;
use common::config::services::GatewayConfig;
use common::config::types::traits::MinKnownConfigTrait;
use common::public_refs::{PublicRefsModule, RemotePublicResolver};
use common::well_known::WellKnownModule;
use ymir::config::types::HostType;
use sea_orm_migration::MigrationTrait;
use ymir::boot::BootstrapServiceTrait;
use ymir::config::traits::CommonConfigTrait;
use ymir::errors::Outcome;
use ymir::module_loader::root_context::RootContext;
use ymir::module_loader::service_composer::ServiceComposer;

use crate::setup::composition::BffModule;

/// Standalone gateway: owns no migrations, but validates tokens against the shared DB.
pub struct GatewayBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for GatewayBoot {
    type Config = GatewayConfig;

    fn banner(big_name: &str) -> String {
        common::info_banner::banner(big_name)
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }

    async fn compose(config: &GatewayConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        // Agreements first, as in the monolith; the gateway holds no URNs of its own.
        let public_refs = PublicRefsModule::new(
            config.common(),
            vec![
                Arc::new(RemotePublicResolver::new(config.contracts().get_host(HostType::Http))),
                Arc::new(RemotePublicResolver::new(config.catalog().get_host(HostType::Http))),
            ],
        );
        Ok(ServiceComposer::new()
            .register(BffModule::compose(config, root))
            .register(public_refs)
            .register(WellKnownModule::new(config.common(), None)?))
    }
}
