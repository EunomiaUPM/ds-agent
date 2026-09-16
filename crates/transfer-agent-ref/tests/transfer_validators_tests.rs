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

use common::validation::codes;
use transfer_agent_ref::entities::protocol::TransferRole::{self, Consumer, Provider, Relay};
use transfer_agent_ref::protocols::dsp::entities::message_types::TransferDSPMessageType::{
    self, TransferCompletionMessage, TransferRequestMessage, TransferStartMessage,
    TransferSuspensionMessage, TransferTerminationMessage,
};
use transfer_agent_ref::protocols::dsp::entities::state::TransferDSPState::{
    self, COMPLETED, REQUESTED, STARTED, SUSPENDED,
};
use transfer_agent_ref::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute::{
    self, ByConsumer, ByProvider, OnRequest,
};
use transfer_agent_ref::protocols::dsp::services::validator::{TransferRules, TransferValidators};

#[test]
fn transfer_rules_state_machine_transitions() {
    // Legal transitions
    assert!(TransferRules::state_transition(None, &TransferRequestMessage, "state").is_ok());
    assert!(TransferRules::state_transition(Some(&REQUESTED), &TransferStartMessage, "state").is_ok());
    assert!(TransferRules::state_transition(Some(&SUSPENDED), &TransferStartMessage, "state").is_ok());
    assert!(TransferRules::state_transition(Some(&STARTED), &TransferCompletionMessage, "state").is_ok());
    assert!(TransferRules::state_transition(Some(&STARTED), &TransferSuspensionMessage, "state").is_ok());
    assert!(TransferRules::state_transition(Some(&REQUESTED), &TransferTerminationMessage, "state").is_ok());

    // Illegal transitions
    let err_req = TransferRules::state_transition(Some(&REQUESTED), &TransferRequestMessage, "state").unwrap_err();
    assert_eq!(err_req.code(), Some(codes::NOT_ALLOWED));

    let err_start = TransferRules::state_transition(Some(&COMPLETED), &TransferStartMessage, "state").unwrap_err();
    assert_eq!(err_start.code(), Some(codes::NOT_ALLOWED));

    let err_none_start = TransferRules::state_transition(None, &TransferStartMessage, "state").unwrap_err();
    assert_eq!(err_none_start.code(), Some(codes::NOT_ALLOWED));
}

#[test]
fn transfer_rules_role_gating() {
    assert!(TransferRules::role_for_message(&Provider, &TransferRequestMessage, "role").is_ok());
    assert!(TransferRules::role_for_message(&Consumer, &TransferRequestMessage, "role").is_err());
    assert!(TransferRules::role_for_message(&Consumer, &TransferStartMessage, "role").is_ok());
    assert!(TransferRules::role_for_message(&Relay, &TransferStartMessage, "role").is_err());
}

#[test]
fn transfer_rules_suspension_semaphore() {
    // A role cannot resume a transfer it suspended itself
    assert!(TransferRules::semaphore(&ByConsumer, &TransferStartMessage, &Consumer, "sem").is_err());
    assert!(TransferRules::semaphore(&ByProvider, &TransferStartMessage, &Provider, "sem").is_err());

    // The counterparty can resume
    assert!(TransferRules::semaphore(&ByConsumer, &TransferStartMessage, &Provider, "sem").is_ok());
    assert!(TransferRules::semaphore(&ByProvider, &TransferStartMessage, &Consumer, "sem").is_ok());

    // First start always passes
    assert!(TransferRules::semaphore(&OnRequest, &TransferStartMessage, &Consumer, "sem").is_ok());
    assert!(TransferRules::semaphore(&OnRequest, &TransferStartMessage, &Provider, "sem").is_ok());
}

#[test]
fn transfer_rules_data_address_format() {
    assert!(TransferRules::data_address_format(true, &Provider, &OnRequest, "dataAddress").is_ok());
    assert!(TransferRules::data_address_format(true, &Consumer, &OnRequest, "dataAddress").is_ok());
    assert!(TransferRules::data_address_format(true, &Consumer, &ByProvider, "dataAddress").is_err());
    assert!(TransferRules::data_address_format(false, &Consumer, &ByProvider, "dataAddress").is_ok());
}

#[test]
fn transfer_rules_pids_and_uri_correlation() {
    // URI and body PID correlation
    assert!(TransferRules::uri_and_pid("urn:uuid:1", Some("urn:uuid:1"), None, &Consumer, "uri").is_ok());
    assert!(TransferRules::uri_and_pid("urn:uuid:1", None, Some("urn:uuid:1"), &Provider, "uri").is_ok());
    assert!(TransferRules::uri_and_pid("urn:uuid:1", Some("urn:uuid:2"), None, &Consumer, "uri").is_err());

    // Stored PID correlation
    assert!(TransferRules::correlation(Some("c"), Some("p"), Some("c"), Some("p"), "pids").is_ok());
    assert!(TransferRules::correlation(Some("c"), Some("p"), Some("c"), Some("other"), "pids").is_err());
}

#[test]
fn registries_are_populated_for_dsp_lifecycle() {
    let dsp_reg = TransferValidators::dsp_registry();
    let rpc_reg = TransferValidators::rpc_registry();
    let edge_reg = TransferValidators::edge_registry();

    for msg in [
        TransferRequestMessage,
        TransferStartMessage,
        TransferCompletionMessage,
        TransferSuspensionMessage,
        TransferTerminationMessage,
    ] {
        assert!(dsp_reg.has_key(&msg));
        assert!(rpc_reg.has_key(&msg));
        assert!(edge_reg.has_key(&msg));
    }
}
