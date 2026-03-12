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

pub(crate) mod rpc;
pub(crate) mod types;
pub(crate) mod step_trait;
pub(crate) mod step_request;
pub(crate) mod step_start;
pub(crate) mod step_suspension;
pub(crate) mod step_completion;
pub(crate) mod step_termination;

use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferCompletionMessageDto, RpcTransferMessageDto, RpcTransferRequestMessageDto,
    RpcTransferStartMessageDto, RpcTransferSuspensionMessageDto, RpcTransferTerminationMessageDto,
};

#[async_trait::async_trait]
pub trait RPCOrchestratorTrait: Send + Sync + 'static {
    async fn setup_transfer_request(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferRequestMessageDto>>;
    async fn setup_transfer_start(
        &self,
        input: &RpcTransferStartMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferStartMessageDto>>;
    async fn setup_transfer_suspension(
        &self,
        input: &RpcTransferSuspensionMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferSuspensionMessageDto>>;
    async fn setup_transfer_completion(
        &self,
        input: &RpcTransferCompletionMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferCompletionMessageDto>>;
    async fn setup_transfer_termination(
        &self,
        input: &RpcTransferTerminationMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferTerminationMessageDto>>;
}
