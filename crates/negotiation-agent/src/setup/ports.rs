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

use common::config::services::ContractsConfig;
use common::config::services::traits::ContractsConfigTrait;
use common::facades::AuthPorts;
use common::facades::catalog::CatalogFacadeTrait;
use common::facades::catalog::remote::CatalogRemoteFacade;
use ymir::module_loader::root_context::RootContext;

/// Ports the negotiation agent consumes: auth and the catalog's published contract.
#[derive(Clone)]
pub struct NegotiationPorts {
    pub auth: AuthPorts,
    pub catalog: Arc<dyn CatalogFacadeTrait>,
}

impl NegotiationPorts {
    /// Microservices: auth and catalog are reached through their APIs.
    pub fn remote(config: &ContractsConfig, _root: &RootContext) -> Self {
        Self {
            auth: AuthPorts::remote(config.ssi_auth()),
            catalog: Arc::new(CatalogRemoteFacade::new(config.catalog())),
        }
    }

    /// Monolith: the auth module's adapters and the catalog module's contract, in-process.
    pub fn local(auth: AuthPorts, catalog: Arc<dyn CatalogFacadeTrait>) -> Self {
        Self { auth, catalog }
    }

    pub fn auth(&self) -> &AuthPorts {
        &self.auth
    }
}
