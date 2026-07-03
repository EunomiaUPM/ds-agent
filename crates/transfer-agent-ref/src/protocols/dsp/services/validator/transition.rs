//! The DSP transfer state machine, ported from the mature
//! `validate_state_transition`. Pure functions over the domain enums — the
//! current state is parsed at the edge (see [`current_state`]).

use std::str::FromStr;

use ymir::errors::{Errors, Outcome};

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_common::TransferContextProcessSlot;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

/// The current state of the process this message targets, parsed at the edge.
/// `None` for a brand-new process (the `New` slot) — there is no prior state.
pub fn current_state(process: &TransferContextProcessSlot) -> Outcome<Option<TransferDSPState>> {
    match process {
        TransferContextProcessSlot::New { .. } => Ok(None),
        TransferContextProcessSlot::Existing(p) => {
            TransferDSPState::from_str(p.state().0.as_str()).map(Some)
        }
    }
}

/// Role × message: who may *receive* what. A provider receives every message; a
/// consumer receives all but `TransferRequestMessage`; `Relay` is not a transfer
/// peer.
pub fn validate_role_for_message(
    role: &TransferRole,
    message: &TransferDSPMessageType,
) -> Outcome<()> {
    use TransferDSPMessageType::TransferRequestMessage;
    match (role, message) {
        (TransferRole::Provider, _) => Ok(()),
        (TransferRole::Consumer, TransferRequestMessage) => Err(Errors::parse(
            "only a Provider may receive a TransferRequestMessage",
            None,
        )),
        (TransferRole::Consumer, _) => Ok(()),
        (TransferRole::Relay, _) => Err(Errors::parse("Relay is not a transfer peer", None)),
    }
}

/// State × message: the DSP state machine. `current == None` is a brand-new
/// process, where only `TransferRequestMessage` is valid.
pub fn validate_state_transition(
    current: Option<&TransferDSPState>,
    message: &TransferDSPMessageType,
) -> Outcome<()> {
    use TransferDSPMessageType::*;
    use TransferDSPState::*;
    let reject = || {
        let from = current
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<new>".to_string());
        Err(Errors::parse(
            format!("{message} is not allowed from state {from}"),
            None,
        ))
    };
    match (message, current) {
        // request mints the process; only valid when none exists yet
        (TransferRequestMessage, None) => Ok(()),
        (TransferRequestMessage, Some(_)) => reject(),
        // start: from Requested, or resuming a Suspended one
        (TransferStartMessage, Some(REQUESTED | SUSPENDED)) => Ok(()),
        (TransferStartMessage, _) => reject(),
        // completion: from Started, or a Suspended flow being wound up
        (TransferCompletionMessage, Some(STARTED | SUSPENDED)) => Ok(()),
        (TransferCompletionMessage, _) => reject(),
        // suspension: only an active (Started) transfer
        (TransferSuspensionMessage, Some(STARTED)) => Ok(()),
        (TransferSuspensionMessage, _) => reject(),
        // termination: from Requested, Started or Suspended
        (TransferTerminationMessage, Some(REQUESTED | STARTED | SUSPENDED)) => Ok(()),
        (TransferTerminationMessage, _) => reject(),
        // not lifecycle transitions
        (TransferProcess | TransferError, _) => Err(Errors::parse(
            format!("{message} is not a lifecycle transition"),
            None,
        )),
    }
}

/// The start↔suspension semaphore: a role cannot resume a transfer *it itself*
/// suspended (the counterparty must). Only checked on `TransferStartMessage`;
/// `OnRequest` (the first start) always passes.
pub fn validate_state_attribute_transition(
    attribute: &TransferDSPStateAttribute,
    message: &TransferDSPMessageType,
    role: &TransferRole,
) -> Outcome<()> {
    use TransferDSPMessageType::TransferStartMessage;
    use TransferDSPStateAttribute::{ByConsumer, ByProvider, OnRequest};
    use TransferRole::{Consumer, Provider};
    match message {
        TransferStartMessage => match (attribute, role) {
            (OnRequest, _) => Ok(()),
            (ByConsumer, Consumer) | (ByProvider, Provider) => Err(Errors::parse(
                "cannot resume a transfer suspended by your own role",
                None,
            )),
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TransferDSPMessageType::*;
    use TransferDSPState::*;

    #[test]
    fn state_machine_legal_and_illegal() {
        // legal
        assert!(validate_state_transition(None, &TransferRequestMessage).is_ok());
        assert!(validate_state_transition(Some(&REQUESTED), &TransferStartMessage).is_ok());
        assert!(validate_state_transition(Some(&SUSPENDED), &TransferStartMessage).is_ok());
        assert!(validate_state_transition(Some(&STARTED), &TransferCompletionMessage).is_ok());
        assert!(validate_state_transition(Some(&STARTED), &TransferSuspensionMessage).is_ok());
        assert!(validate_state_transition(Some(&REQUESTED), &TransferTerminationMessage).is_ok());
        // illegal
        assert!(validate_state_transition(Some(&REQUESTED), &TransferRequestMessage).is_err());
        assert!(validate_state_transition(Some(&COMPLETED), &TransferStartMessage).is_err());
        assert!(validate_state_transition(Some(&REQUESTED), &TransferSuspensionMessage).is_err());
        assert!(validate_state_transition(Some(&TERMINATED), &TransferTerminationMessage).is_err());
        assert!(validate_state_transition(None, &TransferStartMessage).is_err());
    }

    #[test]
    fn role_gate() {
        assert!(
            validate_role_for_message(&TransferRole::Provider, &TransferRequestMessage).is_ok()
        );
        assert!(
            validate_role_for_message(&TransferRole::Consumer, &TransferRequestMessage).is_err()
        );
        assert!(validate_role_for_message(&TransferRole::Consumer, &TransferStartMessage).is_ok());
        assert!(validate_role_for_message(&TransferRole::Relay, &TransferStartMessage).is_err());
    }

    #[test]
    fn suspension_semaphore() {
        use TransferDSPStateAttribute::*;
        // a role can't resume what it suspended…
        assert!(
            validate_state_attribute_transition(
                &ByConsumer,
                &TransferStartMessage,
                &TransferRole::Consumer
            )
            .is_err()
        );
        assert!(
            validate_state_attribute_transition(
                &ByProvider,
                &TransferStartMessage,
                &TransferRole::Provider
            )
            .is_err()
        );
        // …but the counterparty can
        assert!(
            validate_state_attribute_transition(
                &ByConsumer,
                &TransferStartMessage,
                &TransferRole::Provider
            )
            .is_ok()
        );
        // the first start always passes
        assert!(
            validate_state_attribute_transition(
                &OnRequest,
                &TransferStartMessage,
                &TransferRole::Consumer
            )
            .is_ok()
        );
        // other messages aren't gated by the semaphore
        assert!(
            validate_state_attribute_transition(
                &ByConsumer,
                &TransferCompletionMessage,
                &TransferRole::Consumer
            )
            .is_ok()
        );
    }
}
