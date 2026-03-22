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
use crate::protocols::dsp::orchestrator::rpc::types::RpcNegotiationProcessMessageTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationEventType, NegotiationProcessMessageTrait, NegotiationProcessMessageType,
    NegotiationProcessMessageWrapper, NegotiationProcessState,
};
use common::config::types::roles::RoleConfig;
use common::dsp_common::odrl::{ContractRequestMessageOfferTypes, OdrlAgreement};
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use tracing::error;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub trait OrchestrationHelpers: Send + Sync + 'static {
    fn convert_string_to_urn(&self, uri_id: &String) -> Outcome<Urn> {
        Urn::from_str(uri_id.as_str()).map_err(|_e| {
            let err = CommonErrors::parse_new("Invalid URN URN. The URN URN is malformed.");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })
    }
    fn convert_str_to_urn(&self, str: &str) -> Outcome<Urn> {
        let urn = Urn::from_str(str).map_err(|_err| {
            let err = CommonErrors::parse_new("Not able to parse string into Urn");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(urn)
    }
    fn parse_identifier_into_role(&self, identifier: &str) -> Outcome<RoleConfig> {
        match identifier.to_lowercase().as_str() {
            "consumerpid" => Ok(RoleConfig::Consumer),
            "providerpid" => Ok(RoleConfig::Provider),
            _ => {
                let err = CommonErrors::parse_new("Not able to parse indentifier into role");
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string().as_str(), None));
            }
        }
    }
    fn parse_role_into_identifier(&self, role: &RoleConfig) -> Outcome<&str> {
        match role {
            RoleConfig::Provider => Ok("providerPid"),
            RoleConfig::Consumer => Ok("consumerPid"),
            _ => {
                let err = CommonErrors::parse_new("Not able to parse indentifier into role");
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string().as_str(), None));
            }
        }
    }
    fn get_pid_by_role(&self, dto: &NegotiationProcessDto, role: RoleConfig) -> Outcome<Urn> {
        let role_as_identifier = self.parse_role_into_identifier(&role)?;
        let pid = dto.identifiers.get(role_as_identifier).ok_or_else(|| {
            let err = CommonErrors::parse_new("There is no such a identifier, role is mandatory.");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        let urn = self.convert_string_to_urn(pid)?;
        Ok(urn)
    }
    fn create_entity_urn(&self, entity: &str) -> Outcome<Urn> {
        let urn = Urn::from_str(format!("urn:{}:{}", entity, uuid::Uuid::new_v4()).as_str())
            .map_err(|_err| {
                let err = CommonErrors::parse_new("Not able to create Urn");
                error!("{}", err.log());
                Errors::parse(err.to_string().as_str(), None)
            })?;
        Ok(urn)
    }

    fn get_rpc_consumer_pid_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<Urn> {
        let field = rpc.get_consumer_pid().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract consumer_pid from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_provider_pid_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<Urn> {
        let field = rpc.get_provider_pid().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract provider_pid from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_associated_agent_peer_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<String> {
        let field = rpc.get_associated_agent_peer().ok_or_else(|| {
            let err = CommonErrors::parse_new(
                "Not able to extract associated_agent_peer from rpc message",
            );
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_offer_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<ContractRequestMessageOfferTypes> {
        let field = rpc.get_offer().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract offer from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_agreement_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<OdrlAgreement> {
        let field = rpc.get_agreement().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract agreement from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_provider_address_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<String> {
        let field = rpc.get_provider_address().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("Not able to extract provider_address from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_callback_address_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<String> {
        let field = rpc.get_callback_address().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("Not able to extract callback_address from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_event_type_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationEventType> {
        let field = rpc.get_event_type().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract event_type from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_error_code_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<String> {
        let field = rpc.get_error_code().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract error_code from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_error_reason_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<Vec<String>> {
        let field = rpc.get_error_reason().ok_or_else(|| {
            let err = CommonErrors::parse_new("Not able to extract error_reason from rpc message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_rpc_message_safely(
        &self,
        rpc: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessMessageType> {
        let field = rpc.get_message();
        Ok(field)
    }
    fn get_dsp_consumer_pid_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<Urn> {
        let field = payload.get_consumer_pid().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("not able to extract consumer_pid from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_provider_pid_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<Urn> {
        let field = payload.get_provider_pid().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("not able to extract provider_pid from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_offer_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<ContractRequestMessageOfferTypes> {
        let field = payload.get_offer().ok_or_else(|| {
            let err = CommonErrors::parse_new("not able to extract offer from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_agreement_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<OdrlAgreement> {
        let field = payload.get_agreement().ok_or_else(|| {
            let err = CommonErrors::parse_new("not able to extract agreement from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_event_type_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationEventType> {
        let field = payload.get_event_type().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("not able to extract event_type from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_callback_address_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<String> {
        let field = payload.get_callback_address().ok_or_else(|| {
            let err = CommonErrors::parse_new(
                "not able to extract callback_address from payload message",
            );
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_error_code_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<String> {
        let field = payload.get_error_code().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("not able to extract error_code from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_error_reason_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<Vec<String>> {
        let field = payload.get_error_reason().ok_or_else(|| {
            let err =
                CommonErrors::parse_new("not able to extract error_reason from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_state_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessState> {
        let field = payload.get_state().ok_or_else(|| {
            let err = CommonErrors::parse_new("not able to extract state from payload message");
            error!("{}", err.log());
            Errors::parse(err.to_string().as_str(), None)
        })?;
        Ok(field)
    }
    fn get_dsp_message_safely(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessMessageType> {
        let field = payload.get_message();
        Ok(field)
    }
}
