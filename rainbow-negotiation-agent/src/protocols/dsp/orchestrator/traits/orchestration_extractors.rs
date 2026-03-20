use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::protocol_types::{
    NegotiationProcessMessageTrait, NegotiationProcessMessageType, NegotiationProcessState,
};
use async_trait::async_trait;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::errors::{CommonErrors, ErrorLog};
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
