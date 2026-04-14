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

pub(crate) mod protocol;
pub(crate) mod step_completion;
pub(crate) mod step_request;
pub(crate) mod step_start;
pub(crate) mod step_suspension;
pub(crate) mod step_termination;
pub(crate) mod step_trait;

use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferProcessAckDto, TransferProcessMessageWrapper,
    TransferRequestMessageDto, TransferStartMessageDto, TransferSuspensionMessageDto,
    TransferTerminationMessageDto,
};
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ProtocolOrchestratorTrait: Send + Sync + 'static {
    async fn on_get_transfer_process(
        &self,
        id: &String,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>;
    async fn on_transfer_request(
        &self,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
        associated_agent_peer: &str,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)>;
    async fn on_transfer_start(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>;
    async fn on_transfer_suspension(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferSuspensionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>;
    async fn on_transfer_completion(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferCompletionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>;
    async fn on_transfer_termination(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>;
}
