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
use crate::protocols::dsp::protocol_types::{
    TransferProcessMessageType, TransferProcessState, TransferStateAttribute,
};
use rainbow_common::config::types::roles::RoleConfig;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ValidateStateTransition: Send + Sync + 'static {
    // validate role for message type
    // provider can receive: [request, start, suspension, completion, termination]
    // consumer can receive: [start, suspension, completion, termination]
    async fn validate_role_for_message(
        &self,
        role: &RoleConfig,
        message_type: &TransferProcessMessageType,
    ) -> Outcome<()>;
    // validate state transition from state a to b based in DSP state machine
    async fn validate_state_transition(
        &self,
        current_state: &TransferProcessState,
        message_type: &TransferProcessMessageType,
    ) -> Outcome<()>;
    // logical semaphore for avoiding consumer to start provider's suspension and viceversa
    async fn validate_state_attribute_transition(
        &self,
        current_state: &TransferProcessState,
        current_state_attribute: &TransferStateAttribute,
        message_type: &TransferProcessMessageType,
        role: &RoleConfig,
    ) -> Outcome<()>;
}
