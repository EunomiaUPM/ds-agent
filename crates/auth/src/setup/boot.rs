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

use crate::setup::composition::AuthModule;
use common::boot::BootstrapServiceTrait;
use common::config::services::SsiAuthConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use common::well_known::WellKnownModule;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::CommonConfigTrait;
use ymir::errors::Outcome;

/// Standalone SSI auth agent: wallet, GNAP gatekeeper, verifier and issuer.
pub struct AuthBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for AuthBoot {
    type Config = SsiAuthConfig;

    fn banner(big_name: &str) -> String {
        common::info_banner::banner(big_name)
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        AuthModule::migrations()
    }

    async fn compose(config: &SsiAuthConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let auth = AuthModule::compose(config, root, None).await?;
        let ports = auth.local_ports();
        Ok(ServiceComposer::new()
            .register(auth)
            .register(WellKnownModule::new(
                config.common(),
                Some(ports.mates.clone()),
            )?))
    }
}
