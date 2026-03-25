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
    NegotiationProcessMessageType, NegotiationProcessState,
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
}
#[async_trait::async_trait]
impl ValidateStateTransition for ValidatedStateTransitionServiceForRcp {
    async fn validate_role_for_message(
        &self,
        _role: &RoleConfig,
        _message_type: &NegotiationProcessMessageType,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn validate_state_transition(
        &self,
        _current_state: &NegotiationProcessState,
        _message_type: &NegotiationProcessMessageType,
    ) -> Outcome<()> {
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
