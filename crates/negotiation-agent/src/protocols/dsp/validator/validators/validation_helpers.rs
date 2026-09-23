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

use common::dsp_common::DspActor;

use crate::protocols::dsp::persistence::process_resolver::NegotiationProcessResolver;
use crate::protocols::dsp::protocol_types::{
    NegotiationProcessMessageTrait, NegotiationProcessState,
};
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use crate::services::negotiation_process::views::NegotiationProcessView;
use common::config::types::roles::RoleConfig;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct ValidationHelperService {
    resolver: Arc<NegotiationProcessResolver>,
}
impl ValidationHelperService {
    pub fn new(resolver: Arc<NegotiationProcessResolver>) -> Self {
        Self { resolver }
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
        actor: &DspActor,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView> {
        let consumer_pid = payload.get_consumer_pid().ok_or_else(|| {
            Errors::parse("Not a valid DSP payload, consumer_pid is mandatory.", None)
        })?;
        self.resolver.resolve(&consumer_pid, actor).await
    }

    async fn get_current_dto_from_payload_by_provider(
        &self,
        actor: &DspActor,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView> {
        let consumer_pid = payload.get_provider_pid().ok_or_else(|| {
            Errors::parse("Not a valid DSP payload, provider_pid is mandatory.", None)
        })?;
        self.resolver.resolve(&consumer_pid, actor).await
    }

    async fn get_pid_by_role(
        &self,
        dto: &NegotiationProcessView,
        role: &RoleConfig,
    ) -> Outcome<Urn> {
        let role_as_identifier = self.parse_role_into_identifier(&role).await?;
        let pid = dto.identifiers.get(role_as_identifier).ok_or_else(|| {
            Errors::parse("There is no such a identifier, role is mandatory.", None)
        })?;
        let urn = self.parse_urn(pid).await?;
        Ok(urn)
    }

    async fn get_role_from_dto(&self, dto: &NegotiationProcessView) -> Outcome<RoleConfig> {
        let role = &dto.inner.role;
        let role = role
            .parse::<RoleConfig>()
            .map_err(|_| Errors::parse("Not able to parse role", None))?;
        Ok(role)
    }

    async fn get_state_from_dto(
        &self,
        dto: &NegotiationProcessView,
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

    async fn get_state_attribute_from_dto(&self, dto: &NegotiationProcessView) -> Outcome<String> {
        Ok("state_attribute".to_string())
    }
}
