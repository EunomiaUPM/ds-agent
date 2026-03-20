/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use std::sync::Arc;

pub struct OrchestratorService {
    protocol_service: Arc<dyn ProtocolOrchestratorTrait>,
    rpc_service: Arc<dyn RPCOrchestratorTrait>,
}

impl OrchestratorService {
    pub fn new(
        protocol_service: Arc<dyn ProtocolOrchestratorTrait>,
        rpc_service: Arc<dyn RPCOrchestratorTrait>,
    ) -> OrchestratorService {
        Self { protocol_service, rpc_service }
    }
}

impl OrchestratorTrait for OrchestratorService {
    fn get_protocol_service(&self) -> Arc<dyn ProtocolOrchestratorTrait> {
        self.protocol_service.clone()
    }

    fn get_rpc_service(&self) -> Arc<dyn RPCOrchestratorTrait> {
        self.rpc_service.clone()
    }
}
