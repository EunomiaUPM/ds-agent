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

use common::boot::BootstrapServiceTrait;
use common::config::services::TransferConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
// The built-in OAuth crate is disabled (identity from Keycloak or the static user).
// use oauth::setup::{AdminSeeder, OAuthModule};
use sea_orm_migration::MigrationTrait;
use ymir::errors::Outcome;

use crate::setup::{TransferAgentModule, TransferPorts};

/// Standalone transfer agent: its own module; identity comes from the root context.
pub struct TransferBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for TransferBoot {
    type Config = TransferConfig;

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // [OAuthModule::migrations(), TransferAgentModule::migrations()]
        //     .into_iter()
        //     .flatten()
        //     .collect()
        TransferAgentModule::migrations()
    }

    async fn compose(config: &TransferConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let ports = TransferPorts::remote(config, root).await?;
        Ok(ServiceComposer::new()
            // .register(OAuthModule::compose(config.common(), root, None))
            .register(TransferAgentModule::compose(config, root, None, &ports))
            .with_auth_ports(ports.auth.clone()))
    }

    // The OAuth admin seeder went with the built-in OAuth crate.
    // async fn seeders(
    //     config: &TransferConfig,
    //     root: &RootContext,
    // ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
    //     Ok(vec![Box::new(AdminSeeder::new(
    //         root.db.clone(),
    //         config.common(),
    //     ))])
    // }
}
