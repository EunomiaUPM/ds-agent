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

//! TransferRules and the registries of the inbound DSP flow.

use common::validation::codes;
use transfer_agent::entities::protocol::TransferRole::{Consumer, Provider, Relay};
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType::{
    TransferCompletionMessage, TransferRequestMessage, TransferStartMessage,
    TransferSuspensionMessage, TransferTerminationMessage,
};
use transfer_agent::protocols::dsp::entities::state::TransferDSPState::{
    COMPLETED, REQUESTED, STARTED, SUSPENDED,
};
use transfer_agent::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute::{
    ByConsumer, ByProvider, OnRequest,
};
use transfer_agent::protocols::dsp::services::validator::{TransferDspRules, TransferRules};

/// Legal transitions
#[test]
fn transfer_rules_state_machine_transitions() {
    assert!(TransferRules::state_transition(None, &TransferRequestMessage, "state").is_ok());
    assert!(
        TransferRules::state_transition(Some(&REQUESTED), &TransferStartMessage, "state").is_ok()
    );
    assert!(
        TransferRules::state_transition(Some(&SUSPENDED), &TransferStartMessage, "state").is_ok()
    );
    assert!(
        TransferRules::state_transition(Some(&STARTED), &TransferCompletionMessage, "state")
            .is_ok()
    );
    assert!(
        TransferRules::state_transition(Some(&STARTED), &TransferSuspensionMessage, "state")
            .is_ok()
    );
    assert!(TransferRules::state_transition(
        Some(&REQUESTED),
        &TransferTerminationMessage,
        "state"
    )
    .is_ok());

    // Illegal transitions
    let err_req =
        TransferRules::state_transition(Some(&REQUESTED), &TransferRequestMessage, "state")
            .unwrap_err();
    assert_eq!(err_req.code(), Some(codes::NOT_ALLOWED));

    let err_start =
        TransferRules::state_transition(Some(&COMPLETED), &TransferStartMessage, "state")
            .unwrap_err();
    assert_eq!(err_start.code(), Some(codes::NOT_ALLOWED));

    let err_none_start =
        TransferRules::state_transition(None, &TransferStartMessage, "state").unwrap_err();
    assert_eq!(err_none_start.code(), Some(codes::NOT_ALLOWED));
}

/// Only a provider receives a request, only a consumer a start, and a relay neither.
#[test]
fn transfer_rules_role_gating() {
    assert!(TransferRules::role_for_message(&Provider, &TransferRequestMessage, "role").is_ok());
    assert!(TransferRules::role_for_message(&Consumer, &TransferRequestMessage, "role").is_err());
    assert!(TransferRules::role_for_message(&Consumer, &TransferStartMessage, "role").is_ok());
    assert!(TransferRules::role_for_message(&Relay, &TransferStartMessage, "role").is_err());
}

/// A provider receives a start only to resume a suspension; a consumer receives the first one
#[test]
fn provider_receives_start_only_to_resume() {
    assert!(TransferRules::start_received_in(&Provider, Some(&REQUESTED), "state").is_err());
    assert!(TransferRules::start_received_in(&Provider, Some(&SUSPENDED), "state").is_ok());
    assert!(TransferRules::start_received_in(&Consumer, Some(&REQUESTED), "state").is_ok());
    assert!(TransferRules::start_received_in(&Relay, Some(&SUSPENDED), "state").is_err());
}

/// Inbound semaphore (`role` is the receiver): only the party that suspended the transfer may
/// send `TransferStartMessage` to resume it.
#[test]
fn transfer_rules_suspension_semaphore() {
    // Reject when the counterparty tries to resume a transfer suspended by us (the receiver)
    assert!(
        TransferRules::semaphore(&ByConsumer, &TransferStartMessage, &Consumer, "sem").is_err()
    );
    assert!(
        TransferRules::semaphore(&ByProvider, &TransferStartMessage, &Provider, "sem").is_err()
    );

    // Accept when the party that suspended the transfer sends TransferStartMessage to us
    assert!(TransferRules::semaphore(&ByConsumer, &TransferStartMessage, &Provider, "sem").is_ok());
    assert!(TransferRules::semaphore(&ByProvider, &TransferStartMessage, &Consumer, "sem").is_ok());

    // First start always passes
    assert!(TransferRules::semaphore(&OnRequest, &TransferStartMessage, &Consumer, "sem").is_ok());
    assert!(TransferRules::semaphore(&OnRequest, &TransferStartMessage, &Provider, "sem").is_ok());
}

/// A start may carry the pull endpoint to the consumer, never to the provider.
#[test]
fn transfer_rules_data_address_format() {
    assert!(TransferRules::data_address_format(true, &Consumer, "dataAddress").is_ok());
    assert!(
        TransferRules::data_address_format(true, &Consumer, "dataAddress").is_ok(),
        "a resumed pull gets the endpoint again"
    );
    assert!(TransferRules::data_address_format(true, &Provider, "dataAddress").is_err());
    assert!(TransferRules::data_address_format(false, &Provider, "dataAddress").is_ok());
}

/// URI and body PID correlation
#[test]
fn transfer_rules_pids_and_uri_correlation() {
    assert!(
        TransferRules::uri_and_pid("urn:uuid:1", Some("urn:uuid:1"), None, &Consumer, "uri")
            .is_ok()
    );
    assert!(
        TransferRules::uri_and_pid("urn:uuid:1", None, Some("urn:uuid:1"), &Provider, "uri")
            .is_ok()
    );
    assert!(
        TransferRules::uri_and_pid("urn:uuid:1", Some("urn:uuid:2"), None, &Consumer, "uri")
            .is_err()
    );
}

/// Every registry has a validator for each DSP transfer message.
#[test]
fn registries_are_populated_for_dsp_lifecycle() {
    let dsp_reg = TransferDspRules::fact_rules();
    let edge_reg = TransferDspRules::input_rules();

    for msg in [
        TransferRequestMessage,
        TransferStartMessage,
        TransferCompletionMessage,
        TransferSuspensionMessage,
        TransferTerminationMessage,
    ] {
        assert!(dsp_reg.has_key(&msg));
        assert!(edge_reg.has_key(&msg));
    }
}
