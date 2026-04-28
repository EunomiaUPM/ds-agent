/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

pub(crate) mod rpc;
pub(crate) mod step_completion;
pub(crate) mod step_request;
pub(crate) mod step_start;
pub(crate) mod step_suspension;
pub(crate) mod step_termination;
pub(crate) mod step_trait;
pub(crate) mod types;

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferCompletionMessageDto, RpcTransferMessageDto, RpcTransferRequestMessageDto,
    RpcTransferStartMessageDto, RpcTransferSuspensionMessageDto, RpcTransferTerminationMessageDto,
};
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait RPCOrchestratorTrait: Send + Sync + 'static {
    async fn setup_transfer_request(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferRequestMessageDto>>;
    async fn setup_transfer_start(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferStartMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferStartMessageDto>>;
    async fn setup_transfer_suspension(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferSuspensionMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferSuspensionMessageDto>>;
    async fn setup_transfer_completion(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferCompletionMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferCompletionMessageDto>>;
    async fn setup_transfer_termination(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferTerminationMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferTerminationMessageDto>>;
}
