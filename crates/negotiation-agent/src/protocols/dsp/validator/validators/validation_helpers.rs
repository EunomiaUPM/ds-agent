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

use crate::entities::negotiation_process::{NegotiationAgentProcessesTrait, NegotiationProcessDto};
use crate::protocols::dsp::protocol_types::{
    NegotiationProcessMessageTrait, NegotiationProcessState,
};
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use common::config::types::roles::RoleConfig;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct ValidationHelperService {
    negotiation_process_service: Arc<dyn NegotiationAgentProcessesTrait>,
}
impl ValidationHelperService {
    pub fn new(negotiation_process_service: Arc<dyn NegotiationAgentProcessesTrait>) -> Self {
        Self {
            negotiation_process_service,
        }
    }
}
#[async_trait::async_trait]
impl ValidationHelpers for ValidationHelperService {
    async fn parse_urn(&self, uri_id: &String) -> Outcome<Urn> {
        Urn::from_str(uri_id.as_str())
            .map_err(|_| Errors::parse("Invalid URN. The URN is malformed.", None))
    }

    async fn parse_identifier_into_role(&self, identifier: &str) -> Outcome<RoleConfig> {
        match identifier {
            "consumerPid" => Ok(RoleConfig::Consumer),
            "providerPid" => Ok(RoleConfig::Provider),
            _ => {
                return Err(Errors::parse(
                    "Not a valid DSP identifiers. Please use 'consumerPid' or 'providerPid'.",
                    None,
                ));
            }
        }
    }

    async fn parse_role_into_identifier(&self, role: &RoleConfig) -> Outcome<&str> {
        match role {
            RoleConfig::Provider => Ok("providerPid"),
            RoleConfig::Consumer => Ok("consumerPid"),
            _ => {
                return Err(Errors::parse(
                    "Not a valid DSP identifiers. Please use 'consumerPid' or 'providerPid'.",
                    None,
                ));
            }
        }
    }

    async fn get_current_dto_from_payload(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let consumer_pid = payload.get_consumer_pid().ok_or_else(|| {
            Errors::parse("Not a valid DSP payload, consumer_pid is mandatory.", None)
        })?;
        let dto = self
            .negotiation_process_service
            .get_negotiation_process_by_key_value(&consumer_pid)
            .await?
            .ok_or_else(|| Errors::parse("A dto should be available at this point", None))?;
        Ok(dto)
    }

    async fn get_current_dto_from_payload_by_provider(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let consumer_pid = payload.get_provider_pid().ok_or_else(|| {
            Errors::parse("Not a valid DSP payload, provider_pid is mandatory.", None)
        })?;
        let dto = self
            .negotiation_process_service
            .get_negotiation_process_by_key_value(&consumer_pid)
            .await?
            .ok_or_else(|| Errors::parse("A dto should be available at this point", None))?;
        Ok(dto)
    }

    async fn get_pid_by_role(
        &self,
        dto: &NegotiationProcessDto,
        role: &RoleConfig,
    ) -> Outcome<Urn> {
        let role_as_identifier = self.parse_role_into_identifier(&role).await?;
        let pid = dto.identifiers.get(role_as_identifier).ok_or_else(|| {
            Errors::parse("There is no such a identifier, role is mandatory.", None)
        })?;
        let urn = self.parse_urn(pid).await?;
        Ok(urn)
    }

    async fn get_role_from_dto(&self, dto: &NegotiationProcessDto) -> Outcome<RoleConfig> {
        let role = &dto.inner.role;
        let role = role
            .parse::<RoleConfig>()
            .map_err(|_| Errors::parse("Not able to parse role", None))?;
        Ok(role)
    }

    async fn get_state_from_dto(
        &self,
        dto: &NegotiationProcessDto,
    ) -> Outcome<NegotiationProcessState> {
        let state = &dto.inner.state;
        let state = state.parse::<NegotiationProcessState>().map_err(|_| {
            Errors::parse(
                "Something is wrong. Seems this process' state is not protocol compliant",
                None,
            )
        })?;
        Ok(state)
    }

    async fn get_state_attribute_from_dto(&self, dto: &NegotiationProcessDto) -> Outcome<String> {
        Ok("state_attribute".to_string())
    }
}
