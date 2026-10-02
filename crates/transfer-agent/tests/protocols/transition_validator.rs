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

//! Transition rules: which role may send each message, and who may resume a suspension.

use transfer_agent::entities::protocol::TransferRole;
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use transfer_agent::protocols::dsp::entities::state::TransferDSPState;
use transfer_agent::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

use TransferDSPMessageType::*;
use TransferDSPState::*;
use TransitionValidator as V;
use transfer_agent::protocols::dsp::services::validator::transition::*;

/// legal
#[test]
fn state_machine_legal_and_illegal() {
    assert!(V::validate_state_transition(None, &TransferRequestMessage).is_ok());
    assert!(V::validate_state_transition(Some(&REQUESTED), &TransferStartMessage).is_ok());
    assert!(V::validate_state_transition(Some(&SUSPENDED), &TransferStartMessage).is_ok());
    assert!(V::validate_state_transition(Some(&STARTED), &TransferCompletionMessage).is_ok());
    assert!(V::validate_state_transition(Some(&STARTED), &TransferSuspensionMessage).is_ok());
    assert!(V::validate_state_transition(Some(&REQUESTED), &TransferTerminationMessage).is_ok());
    // illegal
    assert!(V::validate_state_transition(Some(&REQUESTED), &TransferRequestMessage).is_err());
    assert!(V::validate_state_transition(Some(&COMPLETED), &TransferStartMessage).is_err());
    assert!(V::validate_state_transition(Some(&REQUESTED), &TransferSuspensionMessage).is_err());
    assert!(V::validate_state_transition(Some(&TERMINATED), &TransferTerminationMessage).is_err());
    assert!(V::validate_state_transition(None, &TransferStartMessage).is_err());
}

/// Only a provider receives a request, only a consumer a start, and a relay neither.
#[test]
fn role_gate() {
    assert!(V::validate_role_for_message(&TransferRole::Provider, &TransferRequestMessage).is_ok());
    assert!(
        V::validate_role_for_message(&TransferRole::Consumer, &TransferRequestMessage).is_err()
    );
    assert!(V::validate_role_for_message(&TransferRole::Consumer, &TransferStartMessage).is_ok());
    assert!(V::validate_role_for_message(&TransferRole::Relay, &TransferStartMessage).is_err());
}

/// Only the counterparty of who suspended may restart, the first start always passes,
/// and other messages are not gated.
#[test]
fn suspension_semaphore() {
    use TransferDSPStateAttribute::*;
    // a role can't resume what it suspended…
    assert!(
        V::validate_state_attribute_transition(
            &ByConsumer,
            &TransferStartMessage,
            &TransferRole::Consumer
        )
        .is_err()
    );
    assert!(
        V::validate_state_attribute_transition(
            &ByProvider,
            &TransferStartMessage,
            &TransferRole::Provider
        )
        .is_err()
    );
    // …but the counterparty can
    assert!(
        V::validate_state_attribute_transition(
            &ByConsumer,
            &TransferStartMessage,
            &TransferRole::Provider
        )
        .is_ok()
    );
    // the first start always passes
    assert!(
        V::validate_state_attribute_transition(
            &OnRequest,
            &TransferStartMessage,
            &TransferRole::Consumer
        )
        .is_ok()
    );
    // other messages aren't gated by the semaphore
    assert!(
        V::validate_state_attribute_transition(
            &ByConsumer,
            &TransferCompletionMessage,
            &TransferRole::Consumer
        )
        .is_ok()
    );
}
