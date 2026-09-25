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

//! Ports the dataplane consumes: connector instances, owned by the catalog agent.

use std::sync::Arc;

use common::config::services::traits::TransferConfigTrait;
use common::config::services::TransferConfig;
use common::module_loader::root_context::RootContext;
use connector::{ConnectorInstanceFacadeTrait, ConnectorInstanceRemoteFacade};

#[derive(Clone)]
pub struct DataplanePorts {
    pub(crate) connector: Arc<dyn ConnectorInstanceFacadeTrait>,
}

impl DataplanePorts {
    /// Microservices: the catalog's connector API with the service token.
    pub fn remote(config: &TransferConfig, root: &RootContext) -> Self {
        Self {
            connector: Arc::new(ConnectorInstanceRemoteFacade::new(
                config.catalog(),
                root.service_client.clone(),
            )),
        }
    }

    /// Monolith: the catalog's in-process adapter.
    pub fn local(connector: Arc<dyn ConnectorInstanceFacadeTrait>) -> Self {
        Self { connector }
    }
}
