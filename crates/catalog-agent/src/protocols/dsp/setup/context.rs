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

//! What the DSP plane uses: the agent context and the two sides of the catalog protocol.

use std::sync::Arc;

use common::well_known::rpc::rpc::WellKnownRPCService;

use crate::protocols::dsp::facades::well_known_rpc_facade::well_known_rpc_facade::WellKnownRPCFacadeForDSProtocol;
use crate::protocols::dsp::services::dsp_catalog::service::DspCatalogService;
use crate::protocols::dsp::services::dsp_catalog::DspCatalogServiceTrait;
use crate::protocols::dsp::services::rpc_catalog::service::RpcCatalogService;
use crate::protocols::dsp::services::rpc_catalog::RpcCatalogServiceTrait;
use crate::setup::context::AppContext;

pub struct DspContext {
    pub app: Arc<AppContext>,
    /// Answering peers.
    pub catalog: Arc<dyn DspCatalogServiceTrait>,
    /// Asking peers for a user.
    pub rpc: Arc<dyn RpcCatalogServiceTrait>,
}

impl DspContext {
    pub fn build(app: Arc<AppContext>) -> Arc<Self> {
        let well_known = Arc::new(WellKnownRPCFacadeForDSProtocol::new(Arc::new(
            WellKnownRPCService::new(app.ports.auth.mates.clone()),
        )));
        Arc::new(Self {
            catalog: Arc::new(DspCatalogService::new(app.clone())),
            rpc: Arc::new(RpcCatalogService::new(app.clone(), well_known)),
            app,
        })
    }
}
