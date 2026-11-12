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

//! Ports the connector consumes: the hosting catalog and an optional guard over active transfers.

use std::sync::{Arc, OnceLock};

use common::facades::catalog::CatalogFacadeTrait;
use urn::Urn;
use ymir::errors::Outcome;

/// Port queried before deleting a connector instance so in-flight transfers are never orphaned.
#[mockall::automock]
#[async_trait::async_trait]
pub trait ActiveTransferGuard: Send + Sync + 'static {
    /// Number of non-terminated dataplane transfers currently referencing `instance_id`.
    async fn count_active_transfers(&self, instance_id: &Urn) -> Outcome<u64>;
}

/// Late-bound [`ActiveTransferGuard`] used in the monolith where `CatalogAgentModule` is composed
/// before `TransferAgentModule` / `DataplaneModule`. Defaults to `0` until bound.
#[derive(Default)]
pub struct LateBoundTransferGuard {
    inner: OnceLock<Arc<dyn ActiveTransferGuard>>,
}

impl LateBoundTransferGuard {
    pub fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    pub fn bind(&self, guard: Arc<dyn ActiveTransferGuard>) {
        let _ = self.inner.set(guard);
    }
}

#[async_trait::async_trait]
impl ActiveTransferGuard for LateBoundTransferGuard {
    async fn count_active_transfers(&self, instance_id: &Urn) -> Outcome<u64> {
        match self.inner.get() {
            Some(guard) => guard.count_active_transfers(instance_id).await,
            None => Ok(0),
        }
    }
}

/// Ports the connector consumes: the hosting catalog and optional active-transfer guard.
#[derive(Clone)]
pub struct ConnectorPorts {
    pub catalog: Arc<dyn CatalogFacadeTrait>,
    pub transfer_guard: Option<Arc<dyn ActiveTransferGuard>>,
}

impl ConnectorPorts {
    /// `catalog` is the hosting catalog's in-process adapter.
    pub fn local(catalog: Arc<dyn CatalogFacadeTrait>) -> Self {
        Self {
            catalog,
            transfer_guard: None,
        }
    }

    /// Attaches an active-transfer guard consulted before deleting connector instances.
    pub fn with_transfer_guard(mut self, guard: Arc<dyn ActiveTransferGuard>) -> Self {
        self.transfer_guard = Some(guard);
        self
    }
}
