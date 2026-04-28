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
use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferProcessMessageWrapper, TransferRequestMessageDto,
    TransferStartMessageDto, TransferSuspensionMessageDto, TransferTerminationMessageDto,
};
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ValidationDspSteps: Send + Sync + 'static {
    async fn on_transfer_request(
        &self,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<()>;
    async fn on_transfer_start(
        &self,
        ctx: &DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<()>;
    async fn on_transfer_completion(
        &self,
        ctx: &DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferCompletionMessageDto>,
    ) -> Outcome<()>;
    async fn on_transfer_suspension(
        &self,
        ctx: &DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferSuspensionMessageDto>,
    ) -> Outcome<()>;
    async fn on_transfer_termination(
        &self,
        ctx: &DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> Outcome<()>;
}
