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

//! Atomic pure rules enforcing DSP 2025-1 transfer state machine and protocol invariants.

use std::str::FromStr;
use common::dsp_common::DspRules;
use common::validation::{codes, violation, Path, Violations};

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_common::TransferContextProcessSlot;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

pub struct TransferRules;

impl TransferRules {
    /// Extract current state from process slot, returning None for a new process.
    pub fn current_state(process: &TransferContextProcessSlot) -> Option<TransferDSPState> {
        match process {
            TransferContextProcessSlot::New { .. } => None,
            TransferContextProcessSlot::Existing(p) => {
                TransferDSPState::from_str(p.state().0.as_str()).ok()
            }
        }
    }

    /// DSP state machine transitions according to DSP 2025-1.
    pub fn state_transition(
        current: Option<&TransferDSPState>,
        message: &TransferDSPMessageType,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        use TransferDSPMessageType::*;
        use TransferDSPState::*;

        let p = path.into();
        let reject = || {
            let from = current
                .map(|s| s.to_string())
                .unwrap_or_else(|| "<new>".to_string());
            Err(violation(
                p.clone(),
                codes::NOT_ALLOWED,
                format!("{message} is not allowed from state {from}"),
            ))
        };

        match (message, current) {
            (TransferRequestMessage, None) => Ok(()),
            (TransferRequestMessage, Some(_)) => reject(),
            (TransferStartMessage, Some(REQUESTED | SUSPENDED)) => Ok(()),
            (TransferStartMessage, _) => reject(),
            (TransferCompletionMessage, Some(STARTED | SUSPENDED)) => Ok(()),
            (TransferCompletionMessage, _) => reject(),
            (TransferSuspensionMessage, Some(STARTED)) => Ok(()),
            (TransferSuspensionMessage, _) => reject(),
            (TransferTerminationMessage, Some(REQUESTED | STARTED | SUSPENDED)) => Ok(()),
            (TransferTerminationMessage, _) => reject(),
            (TransferProcess | TransferError, _) => Err(violation(
                p,
                codes::NOT_ALLOWED,
                format!("{message} is not a lifecycle transition"),
            )),
        }
    }

    /// Verify whether a peer role is legally permitted to receive the message.
    pub fn role_for_message(
        role: &TransferRole,
        message: &TransferDSPMessageType,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        use TransferDSPMessageType::TransferRequestMessage;
        let p = path.into();
        match (role, message) {
            (TransferRole::Provider, _) => Ok(()),
            (TransferRole::Consumer, TransferRequestMessage) => Err(violation(
                p,
                codes::NOT_ALLOWED,
                "only a Provider may receive a TransferRequestMessage",
            )),
            (TransferRole::Consumer, _) => Ok(()),
            (TransferRole::Relay, _) => Err(violation(
                p,
                codes::NOT_ALLOWED,
                "Relay is not a transfer peer",
            )),
        }
    }

    /// Enforce suspension semaphore: a role cannot resume what it suspended itself.
    pub fn semaphore(
        attribute: &TransferDSPStateAttribute,
        message: &TransferDSPMessageType,
        role: &TransferRole,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        use TransferDSPMessageType::TransferStartMessage;
        use TransferDSPStateAttribute::{ByConsumer, ByProvider, OnRequest};
        use TransferRole::{Consumer, Provider};

        let p = path.into();
        match message {
            TransferStartMessage => match (attribute, role) {
                (OnRequest, _) => Ok(()),
                (ByConsumer, Consumer) | (ByProvider, Provider) => Err(violation(
                    p,
                    codes::NOT_ALLOWED,
                    "cannot resume a transfer suspended by your own role",
                )),
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    /// Verify dataAddress presence rules for TransferStart.
    pub fn data_address_format(
        has_data_address: bool,
        role: &TransferRole,
        attribute: &TransferDSPStateAttribute,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        if has_data_address
            && matches!(role, TransferRole::Consumer)
            && !matches!(attribute, TransferDSPStateAttribute::OnRequest)
        {
            return Err(violation(
                p,
                codes::NOT_ALLOWED,
                "dataAddress is only allowed in the first provider TransferStart",
            ));
        }
        Ok(())
    }

    /// Correlate incoming message PIDs with stored process PIDs.
    pub fn correlation(
        process_consumer: Option<&str>,
        process_provider: Option<&str>,
        message_consumer: Option<&str>,
        message_provider: Option<&str>,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        if process_consumer != message_consumer || process_provider != message_provider {
            return Err(violation(
                p,
                codes::NOT_ALLOWED,
                "message pids and process pids are not correlated",
            ));
        }
        Ok(())
    }

    /// Correlate URI path PID with body PID.
    pub fn uri_and_pid(
        uri_id: &str,
        consumer_pid: Option<&str>,
        provider_pid: Option<&str>,
        role: &TransferRole,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        let body_pid = match role {
            TransferRole::Provider => provider_pid,
            TransferRole::Consumer => consumer_pid,
            TransferRole::Relay => {
                return Err(violation(p, codes::NOT_ALLOWED, "Relay has no pid"));
            }
        };

        match body_pid {
            Some(pid) => DspRules::correlate_pids(uri_id, pid, p),
            None => Err(violation(
                p,
                codes::MISSING,
                "body is missing the role's pid",
            )),
        }
    }
}
