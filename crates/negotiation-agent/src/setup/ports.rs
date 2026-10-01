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

//! Ports the negotiation agent consumes from other agents: in-process in the monolith,
//! over HTTP standalone.

use std::sync::Arc;

use catalog_agent::services::odrl_policies::OdrlPolicyServiceTrait;
use common::config::services::ContractsConfig;
use common::config::services::traits::ContractsConfigTrait;
use common::facades::AuthPorts;
use common::module_loader::root_context::RootContext;

use crate::facades::catalog_facade::CatalogFacadeTrait;
use crate::facades::catalog_facade::local::CatalogLocalFacade;
use crate::facades::catalog_facade::remote::CatalogRemoteFacade;

#[derive(Clone)]
pub struct NegotiationPorts {
    pub(crate) auth: AuthPorts,
    pub(crate) catalog: Arc<dyn CatalogFacadeTrait>,
}

impl NegotiationPorts {
    /// Microservices: auth and catalog are reached through their APIs with the service token.
    pub fn remote(config: &ContractsConfig, root: &RootContext) -> Self {
        Self {
            auth: AuthPorts::remote(config.ssi_auth(), root),
            catalog: Arc::new(CatalogRemoteFacade::new(
                config.catalog(),
                root.service_client.clone(),
            )),
        }
    }

    /// Monolith: the auth module's adapters and the catalog's offer service, in-process.
    pub fn local(auth: AuthPorts, offers: Arc<dyn OdrlPolicyServiceTrait>) -> Self {
        Self {
            auth,
            catalog: Arc::new(CatalogLocalFacade::new(offers)),
        }
    }

    pub fn auth(&self) -> &AuthPorts {
        &self.auth
    }
}
