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

use common::dsp_common::odrl::OdrlAgreement;
use common::dsp_common::DspRules;
use common::validation::{Path, Violations, codes, violation};
use std::str::FromStr;

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_common::TransferContextProcessSlot;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

/// Transfer-specific rules for the validator registries.
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

    /// DSP 9.1 state machine, as the table next to the states.
    pub fn state_transition(
        current: Option<&TransferDSPState>,
        message: &TransferDSPMessageType,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        TransferDSPState::MACHINE.check(current, message, path)
    }

    /// The peer must be the agreement's other party: its provider when we consume, and back.
    pub fn agreement_counterparty(
        agreement: &OdrlAgreement,
        our_role: &TransferRole,
        peer_id: &str,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let expected = match our_role {
            TransferRole::Consumer => &agreement.assigner,
            _ => &agreement.assignee,
        };
        if expected == peer_id {
            return Ok(());
        }
        Err(violation(
            path,
            codes::NOT_ALLOWED,
            format!("{peer_id} is not a party to agreement {}", agreement.id),
        ))
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

    /// Whether a role may send the message: only a consumer requests, only a provider starts.
    pub fn role_may_send(
        role: &TransferRole,
        message: &TransferDSPMessageType,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        use TransferDSPMessageType::{TransferRequestMessage, TransferStartMessage};
        let refuse = |why: &'static str| Err(violation(path.into(), codes::NOT_ALLOWED, why));
        match (role, message) {
            (TransferRole::Relay, _) => refuse("Relay is not a transfer peer"),
            (TransferRole::Provider, TransferRequestMessage) => {
                refuse("only a Consumer sends a TransferRequestMessage")
            }
            (TransferRole::Consumer, TransferStartMessage) => {
                refuse("only a Provider sends a TransferStartMessage")
            }
            _ => Ok(()),
        }
    }

    /// DSP 9.1: the first start is the provider's; a consumer only sends one to resume.
    pub fn start_received_in(
        role: &TransferRole,
        current: Option<&TransferDSPState>,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let refuse = |why: &'static str| Err(violation(path.into(), codes::NOT_ALLOWED, why));
        match (role, current) {
            (TransferRole::Provider, Some(TransferDSPState::SUSPENDED)) => Ok(()),
            (TransferRole::Provider, _) => {
                refuse("a Consumer only sends a TransferStartMessage to resume a suspension")
            }
            (TransferRole::Consumer, _) => Ok(()),
            (TransferRole::Relay, _) => refuse("Relay is not a transfer peer"),
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
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        // The provider hands the pull endpoint out on every start, a resume included; it never
        // takes one from the consumer on a start.
        if has_data_address && matches!(role, TransferRole::Provider) {
            return Err(violation(
                p,
                codes::NOT_ALLOWED,
                "a provider does not receive a dataAddress on TransferStart",
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
