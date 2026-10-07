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
use common::config::services::CatalogConfig;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use sea_orm_migration::MigrationTrait;
use ymir::errors::Outcome;

use crate::setup::{CatalogAgentModule, CatalogPorts};

/// Standalone catalog agent; its module seeds the main catalog and policy templates.
pub struct CatalogAgentBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for CatalogAgentBoot {
    type Config = CatalogConfig;

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        CatalogAgentModule::migrations()
    }

    async fn compose(config: &CatalogConfig, root: &RootContext) -> Outcome<ServiceComposer> {
        let ports = CatalogPorts::remote(config, root);
        Ok(ServiceComposer::new()
            .register(CatalogAgentModule::compose(config, root, None, &ports).await?)
            .with_auth_ports(ports.auth.clone()))
    }
}
