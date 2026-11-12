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
use connector::{ConnectorInstanceFacadeTrait, ConnectorInstanceRemoteFacade};
use ymir::module_loader::root_context::RootContext;

use crate::signals::DataplaneSignalSender;

/// Ports the dataplane consumes; today the connector instances and optional control-plane signals.
#[derive(Clone)]
pub struct DataplanePorts {
    pub connector: Arc<dyn ConnectorInstanceFacadeTrait>,
    pub signals: DataplaneSignalSender,
}

impl DataplanePorts {
    /// Microservices: the catalog's connector API.
    pub fn remote(config: &TransferConfig, _root: &RootContext) -> Self {
        Self {
            connector: Arc::new(ConnectorInstanceRemoteFacade::new(config.catalog())),
            signals: DataplaneSignalSender::noop(),
        }
    }

    /// Monolith: the catalog's in-process adapter.
    pub fn local(connector: Arc<dyn ConnectorInstanceFacadeTrait>) -> Self {
        Self {
            connector,
            signals: DataplaneSignalSender::noop(),
        }
    }

    /// Attaches a control-plane signal sender (`Completed` / `Failed`).
    pub fn with_signals(mut self, signals: DataplaneSignalSender) -> Self {
        self.signals = signals;
        self
    }
}
