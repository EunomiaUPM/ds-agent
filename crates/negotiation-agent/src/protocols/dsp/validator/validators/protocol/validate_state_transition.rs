#![allow(unused)]
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
    NegotiationEventType, NegotiationProcessMessageType, NegotiationProcessState,
};
use crate::protocols::dsp::validator::traits::validate_state_transition::ValidateStateTransition;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use log::error;
use common::config::types::roles::RoleConfig;
use common::errors::{CommonErrors, ErrorLog};
use ymir::errors::{Errors, Outcome};
use std::sync::Arc;

pub struct ValidatedStateTransitionServiceForDsp {
    _helpers: Arc<dyn ValidationHelpers>,
}
impl ValidatedStateTransitionServiceForDsp {
    pub fn new(helpers: Arc<dyn ValidationHelpers>) -> Self {
        Self { _helpers: helpers }
    }
}
#[async_trait::async_trait]
impl ValidateStateTransition for ValidatedStateTransitionServiceForDsp {
    async fn validate_role_for_message(
        &self,
        role: &RoleConfig,
        message_type: &NegotiationProcessMessageType,
    ) -> Outcome<()> {
        match (role, message_type) {
            (RoleConfig::Provider, NegotiationProcessMessageType::NegotiationRequestMessage) => {
                Ok(())
            }
            (
                RoleConfig::Provider,
                NegotiationProcessMessageType::NegotiationAgreementVerificationMessage,
            ) => Ok(()),
            (RoleConfig::Consumer, NegotiationProcessMessageType::NegotiationOfferMessage) => {
                Ok(())
            }
            (RoleConfig::Consumer, NegotiationProcessMessageType::NegotiationAgreementMessage) => {
                Ok(())
            }
            (_, NegotiationProcessMessageType::NegotiationEventMessage(_)) => Ok(()),
            (_, NegotiationProcessMessageType::NegotiationTerminationMessage) => Ok(()),
            _ => {
                let err = CommonErrors::parse_new(
                    format!(
                        "This role: {} does not support negotiation process message type: {}",
                        role, message_type
                    )
                    .as_str(),
                );
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string().as_str(), None))
            }
        }
    }

    async fn validate_state_transition(
        &self,
        current_state: &NegotiationProcessState,
        message_type: &NegotiationProcessMessageType,
    ) -> Outcome<()> {
        match message_type {
            NegotiationProcessMessageType::NegotiationRequestMessage => match current_state {
                NegotiationProcessState::Requested => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Offered => {}
                NegotiationProcessState::Accepted => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Agreed => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Verified => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Finalized => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Terminated => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
            },
            NegotiationProcessMessageType::NegotiationOfferMessage => match current_state {
                NegotiationProcessState::Requested => {}
                NegotiationProcessState::Offered => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Accepted => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Agreed => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Verified => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Finalized => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Terminated => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
            },
            NegotiationProcessMessageType::NegotiationEventMessage(event) => match event {
                NegotiationEventType::ACCEPTED => match current_state {
                    NegotiationProcessState::Requested => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Offered => {}
                    NegotiationProcessState::Accepted => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Agreed => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Verified => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Finalized => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Terminated => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                },
                NegotiationEventType::FINALIZED => match current_state {
                    NegotiationProcessState::Requested => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Offered => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Accepted => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Agreed => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Verified => {}
                    NegotiationProcessState::Finalized => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Terminated => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                },
            },
            NegotiationProcessMessageType::NegotiationAgreementMessage => match current_state {
                NegotiationProcessState::Requested => {}
                NegotiationProcessState::Offered => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Accepted => {}
                NegotiationProcessState::Agreed => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Verified => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Finalized => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Terminated => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
            },
            NegotiationProcessMessageType::NegotiationAgreementVerificationMessage => {
                match current_state {
                    NegotiationProcessState::Requested => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Offered => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Accepted => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Agreed => {}
                    NegotiationProcessState::Verified => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Finalized => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                    NegotiationProcessState::Terminated => {
                        validate_state_transition_error_helper(&current_state, message_type)?;
                    }
                }
            }
            NegotiationProcessMessageType::NegotiationTerminationMessage => match current_state {
                NegotiationProcessState::Requested => {}
                NegotiationProcessState::Offered => {}
                NegotiationProcessState::Accepted => {}
                NegotiationProcessState::Agreed => {}
                NegotiationProcessState::Verified => {}
                NegotiationProcessState::Finalized => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
                NegotiationProcessState::Terminated => {
                    validate_state_transition_error_helper(&current_state, message_type)?;
                }
            },
            NegotiationProcessMessageType::NegotiationProcess => {
                let err = CommonErrors::parse_new(
                    "NegotiationProcessMessageType NegotiationProcess is not allowed here",
                );
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string().as_str(), None))
            }
            NegotiationProcessMessageType::NegotiationError => {
                let err = CommonErrors::parse_new(
                    "NegotiationProcessMessageType NegotiationError is not allowed here",
                );
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string().as_str(), None))
            }
        }
        Ok(())
    }
}

fn validate_state_transition_error_helper(
    current_state: &NegotiationProcessState,
    message_type: &NegotiationProcessMessageType,
) -> Outcome<()> {
    let err = CommonErrors::parse_new(
        format!(
            "NegotiationProcessMessageType {} is not allowed here. Current state is {}",
            message_type.to_string(),
            current_state.to_string()
        )
        .as_str(),
    );
    error!("{}", err.log());
    Err(Errors::parse(err.to_string().as_str(), None))
}
