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

pub(crate) mod orchestrator;
pub(crate) mod protocol;
pub(crate) mod rpc;

use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use std::sync::Arc;

pub trait OrchestratorTrait: Send + Sync + 'static {
    fn get_protocol_service(&self) -> Arc<dyn ProtocolOrchestratorTrait>;
    fn get_rpc_service(&self) -> Arc<dyn RPCOrchestratorTrait>;
}
