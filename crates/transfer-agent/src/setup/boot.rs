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
use common::boot::BootstrapServiceTrait;
use common::boot::seeders::BootSeeder;
use common::config::services::{CommonConfig, TransferConfig};
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use oauth::setup::AdminSeeder;
use oauth::setup::OAuthModule;
use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use ymir::errors::Outcome;

use crate::setup::{TransferAgentModule, TransferPorts};

/// Standalone transfer agent: its own module plus the OAuth root it authenticates against.
pub struct TransferBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for TransferBoot {
    type Config = TransferConfig;

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        [OAuthModule::migrations(), TransferAgentModule::migrations()]
            .into_iter()
            .flatten()
            .collect()
    }

    fn validator(common: &CommonConfig, db: DatabaseConnection) -> Arc<dyn OauthTokenValidator> {
        OAuthModule::validator(common, db)
    }

    async fn compose(config: &TransferConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let ports = TransferPorts::remote(config, root).await?;
        Ok(ServiceComposer::new()
            .register(OAuthModule::compose(config.common(), root, None))
            .register(TransferAgentModule::compose(config, root, None, &ports))
            .with_auth_ports(ports.auth.clone()))
    }

    async fn seeders(
        config: &TransferConfig,
        root: &RootContext,
    ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
        Ok(vec![Box::new(AdminSeeder::new(
            root.db.clone(),
            config.common(),
        ))])
    }
}
