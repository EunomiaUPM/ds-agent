/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::protocol_types::{
    NegotiationProcessMessageTrait, NegotiationProcessMessageType, NegotiationProcessState,
};
use async_trait::async_trait;
use common::config::types::roles::RoleConfig;
use common::errors::{CommonErrors, ErrorLog};
use ymir::errors::{Errors, Outcome};

#[async_trait]
pub trait OrchestrationExtractors: Send + Sync {
    fn get_role_from_dto(&self, dto: &NegotiationProcessDto) -> Outcome<RoleConfig> {
        let role = &dto.inner.role;
        let role = role.parse::<RoleConfig>().map_err(|_| Errors::parse("Not able to parse role", None))?;
        Ok(role)
    }

    fn get_state_from_dto(
        &self,
        dto: &NegotiationProcessDto,
    ) -> Outcome<NegotiationProcessState> {
        let state = &dto.inner.state;
        let state = state.parse::<NegotiationProcessState>().map_err(|_e| {
            Errors::parse("Something is wrong. Seems this process' state is not protocol compliant", None)
        })?;
        Ok(state)
    }

    fn get_role_from_message_type(
        &self,
        message: &NegotiationProcessMessageType,
    ) -> Outcome<RoleConfig>;
    fn get_state_from_message_type(
        &self,
        message: &NegotiationProcessMessageType,
    ) -> Outcome<NegotiationProcessState> {
        Ok(message.clone().into())
    }
}
