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

use crate::protocols::dsp::facades::well_known_rpc_facade::WellKnownRPCFacadeTrait;
use common::well_known::rpc::{WellKnownRPCRequest, WellKnownRPCTrait};
use std::sync::Arc;
use ymir::errors::Outcome;

/// Resolves a peer's DSP path with the well-known RPC service in-process, never via our own host.
pub struct WellKnownRPCFacadeForDSProtocol {
    rpc: Arc<dyn WellKnownRPCTrait>,
}

impl WellKnownRPCFacadeForDSProtocol {
    pub fn new(rpc: Arc<dyn WellKnownRPCTrait>) -> Self {
        Self { rpc }
    }
}

#[async_trait::async_trait]
impl WellKnownRPCFacadeTrait for WellKnownRPCFacadeForDSProtocol {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn resolve_dataspace_current_path(&self, input: &WellKnownRPCRequest) -> Outcome<String> {
        Ok(self.rpc.fetch_dataspace_current_path(input).await?.path)
    }
}
