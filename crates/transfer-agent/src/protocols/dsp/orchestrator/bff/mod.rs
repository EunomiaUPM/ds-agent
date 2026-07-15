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

pub(crate) mod bff;

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferMessageDto, RpcTransferRequestMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageWrapper, TransferRequestMessageDto,
};
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait BFFRPCOrchestratorTrait: Send + Sync + 'static {
    /// Consumer-side BFF: sends a TransferRequest with `autoStart = true` so the
    /// Provider automatically responds with a TransferStart message.
    async fn setup_transfer_request_bff_rpc(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferRequestMessageDto>>;

    /// Provider-side BFF: processes an inbound TransferRequest and immediately
    /// chains a TransferStart to the Consumer's callbackAddress.
    async fn on_transfer_request_auto_start(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)>;
}
