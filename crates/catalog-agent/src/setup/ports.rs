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

use std::sync::Arc;

use common::config::services::traits::CatalogConfigTrait;
use common::config::services::CatalogConfig;
use common::facades::AuthPorts;
use connector::ActiveTransferGuard;
use ymir::module_loader::root_context::RootContext;

/// Ports the catalog agent consumes: auth and an optional active-transfer guard for connectors.
#[derive(Clone)]
pub struct CatalogPorts {
    pub auth: AuthPorts,
    pub transfer_guard: Option<Arc<dyn ActiveTransferGuard>>,
}

impl CatalogPorts {
    /// Microservices: the auth agent is reached through its API.
    pub fn remote(config: &CatalogConfig, _root: &RootContext) -> Self {
        Self {
            auth: AuthPorts::remote(config.ssi_auth()),
            transfer_guard: None,
        }
    }

    /// Monolith: the auth module's in-process adapters.
    pub fn local(auth: AuthPorts) -> Self {
        Self {
            auth,
            transfer_guard: None,
        }
    }

    /// Attaches an active-transfer guard consulted before deleting connector instances.
    pub fn with_transfer_guard(mut self, guard: Arc<dyn ActiveTransferGuard>) -> Self {
        self.transfer_guard = Some(guard);
        self
    }

    pub fn auth(&self) -> &AuthPorts {
        &self.auth
    }
}
