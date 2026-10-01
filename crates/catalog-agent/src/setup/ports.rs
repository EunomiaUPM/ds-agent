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

//! Ports the catalog consumes from other agents: in-process in the monolith, over HTTP standalone.

use common::config::services::traits::CatalogConfigTrait;
use common::config::services::CatalogConfig;
use common::facades::AuthPorts;
use common::module_loader::root_context::RootContext;

#[derive(Clone)]
pub struct CatalogPorts {
    pub(crate) auth: AuthPorts,
}

impl CatalogPorts {
    /// Microservices: the auth agent is reached through its API with the service token.
    pub fn remote(config: &CatalogConfig, root: &RootContext) -> Self {
        Self {
            auth: AuthPorts::remote(config.ssi_auth(), root),
        }
    }

    /// Monolith: the auth module's in-process adapters.
    pub fn local(auth: AuthPorts) -> Self {
        Self { auth }
    }

    pub fn auth(&self) -> &AuthPorts {
        &self.auth
    }
}
