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

use crate::protocols::dsp::protocol_types::{
    TransferProcessMessageType, TransferProcessState, TransferStateAttribute,
};
use crate::protocols::dsp::validator::traits::validate_state_transition::ValidateStateTransition;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use common::config::types::roles::RoleConfig;
use common::errors::{CommonErrors, ErrorLog};
use log::error;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct ValidatedStateTransitionServiceForRcp {
    _helpers: Arc<dyn ValidationHelpers>,
}
impl ValidatedStateTransitionServiceForRcp {
    pub fn new(helpers: Arc<dyn ValidationHelpers>) -> Self {
        Self { _helpers: helpers }
    }
    fn validate_state_transition_error_helper(
        &self,
        current_state: &TransferProcessState,
        message_type: &TransferProcessMessageType,
    ) -> Outcome<()> {
        let err = CommonErrors::parse_new(
            format!(
                "TransferProcessMessageType {} is not allowed here. Current state is {}",
                message_type.to_string(),
                current_state.to_string()
            )
            .as_str(),
        );
        error!("{}", err.log());
        Err(Errors::parse(err.to_string(), None))
    }
}

#[async_trait::async_trait]
impl ValidateStateTransition for ValidatedStateTransitionServiceForRcp {
    async fn validate_role_for_message(
        &self,
        role: &RoleConfig,
        message_type: &TransferProcessMessageType,
    ) -> Outcome<()> {
        match (role, message_type) {
            // consumer can send all messages
            (RoleConfig::Consumer, _) => {}
            // provider cannot send TransferRequestMessage
            (RoleConfig::Provider, TransferProcessMessageType::TransferRequestMessage) => {
                let err = CommonErrors::parse_new(
                    "Only Consumer roles are allowed to send TransferProcessMessageType TransferRequestMessage",
                );
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string(), None));
            }
            // consumer can receive all messages but TransferRequestMessage
            (RoleConfig::Provider, _) => {} // each other role should not be allowed
            _ => {}
        }
        Ok(())
    }

    async fn validate_state_transition(
        &self,
        current_state: &TransferProcessState,
        message_type: &TransferProcessMessageType,
    ) -> Outcome<()> {
        match message_type {
            TransferProcessMessageType::TransferRequestMessage => {
                // is not validated since there's no transition
            }
            TransferProcessMessageType::TransferStartMessage => {
                match current_state {
                    TransferProcessState::Requested => {}
                    TransferProcessState::Started => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    TransferProcessState::Terminated => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    TransferProcessState::Completed => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    TransferProcessState::Suspended => {
                        // TODO check if startable if was suspended by same role
                    }
                }
            }
            TransferProcessMessageType::TransferCompletionMessage => match current_state {
                TransferProcessState::Requested => {
                    self.validate_state_transition_error_helper(&current_state, message_type)?;
                }
                TransferProcessState::Started => {}
                TransferProcessState::Terminated => {
                    self.validate_state_transition_error_helper(&current_state, message_type)?;
                }
                TransferProcessState::Completed => {
                    self.validate_state_transition_error_helper(&current_state, message_type)?;
                }
                TransferProcessState::Suspended => {}
            },
            TransferProcessMessageType::TransferSuspensionMessage => {
                match current_state {
                    TransferProcessState::Requested => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    TransferProcessState::Started => {
                        // TODO check if suspendable if was started by same role
                    }
                    TransferProcessState::Terminated => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    TransferProcessState::Completed => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    TransferProcessState::Suspended => {
                        self.validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                }
            }
            TransferProcessMessageType::TransferTerminationMessage => match current_state {
                TransferProcessState::Requested => {}
                TransferProcessState::Started => {}
                TransferProcessState::Terminated => {
                    self.validate_state_transition_error_helper(&current_state, message_type)?;
                }
                TransferProcessState::Completed => {
                    self.validate_state_transition_error_helper(&current_state, message_type)?;
                }
                TransferProcessState::Suspended => {}
            },
            TransferProcessMessageType::TransferProcess => {
                let err = CommonErrors::parse_new(
                    "TransferProcessMessageType TransferProcess is not allowed here",
                );
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string(), None));
            }
            TransferProcessMessageType::TransferError => {
                let err = CommonErrors::parse_new(
                    "TransferProcessMessageType TransferProcess is not allowed here",
                );
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string(), None));
            }
        }
        Ok(())
    }

    async fn validate_state_attribute_transition(
        &self,
        current_state: &TransferProcessState,
        current_state_attribute: &TransferStateAttribute,
        message_type: &TransferProcessMessageType,
        role: &RoleConfig,
    ) -> Outcome<()> {
        match message_type {
            TransferProcessMessageType::TransferStartMessage => match current_state_attribute {
                TransferStateAttribute::OnRequest => {}
                t => match (t, role) {
                    (TransferStateAttribute::ByConsumer, RoleConfig::Provider)
                    | (TransferStateAttribute::ByProvider, RoleConfig::Consumer) => {
                        let err = CommonErrors::parse_new(
                            format!(
                                "TransferProcessMessageType {} is not allowed here. Current state is {} {}",
                                message_type.to_string(),
                                current_state.to_string(),
                                current_state_attribute.to_string()
                            )
                            .as_str(),
                        );
                        error!("{}", err.log());
                        return Err(Errors::parse(err.to_string(), None));
                    }
                    _ => {}
                },
            },
            _ => {}
        };
        Ok(())
    }
}
