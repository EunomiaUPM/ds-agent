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

//! Rules of the inbound DSP flow: shape over the input, then the loaded facts.

use common::dsp_common::DspRules;
use common::validation::{Rule, Stage, ValidationRegistry, Validator, Violations};
use std::str::FromStr;

use crate::protocols::dsp::entities::context_common::TransferContextProcessSlot;
use crate::protocols::dsp::entities::context_dsp::{TransferDspDomain, TransferDspTyped};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType::{
    TransferCompletionMessage, TransferRequestMessage, TransferStartMessage,
    TransferSuspensionMessage, TransferTerminationMessage,
};
use crate::protocols::dsp::entities::protocol_fields::TransferProtocolFields;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;
use crate::protocols::dsp::services::validator::rules::TransferRules;

/// Registries of the inbound DSP flow, and the message shapes the RPC flow reuses.
pub struct TransferDspRules;

impl TransferDspRules {
    /// Inbound DSP shape rules over the extracted fields, before any lookup.
    pub fn input_rules() -> ValidationRegistry<TransferDSPMessageType, TransferDspTyped> {
        let pids = || Self::on_fields(Self::pids_message());
        ValidationRegistry::new("transfer.dsp.input")
            .with_any(Stage::Shape, Self::route_matches_body())
            .with(
                TransferRequestMessage,
                Stage::Shape,
                Self::on_fields(Self::request_message()),
            )
            .with(TransferStartMessage, Stage::Shape, pids())
            .with(TransferCompletionMessage, Stage::Shape, pids())
            .with(TransferSuspensionMessage, Stage::Shape, pids())
            .with(TransferTerminationMessage, Stage::Shape, pids())
    }

    /// Inbound DSP rules over loaded facts: state machine, role, semaphore and correlation.
    pub fn fact_rules() -> ValidationRegistry<TransferDSPMessageType, TransferDspDomain> {
        use TransferDSPMessageType::*;
        let correlated = || {
            Validator::new()
                .rule(Self::transition)
                .rule(Self::correlated)
        };
        ValidationRegistry::new("transfer.dsp.facts")
            .with(
                TransferRequestMessage,
                Stage::Domain,
                Validator::new().rule(Self::transition).rule(Self::role),
            )
            .with(
                TransferStartMessage,
                Stage::Domain,
                correlated()
                    .rule(Self::role)
                    .rule(Self::resume_only)
                    .rule(Self::semaphore)
                    .rule(Self::data_address)
                    .rule(Self::path_matches_pid),
            )
            .with(TransferCompletionMessage, Stage::Domain, correlated())
            .with(TransferSuspensionMessage, Stage::Domain, correlated())
            .with(TransferTerminationMessage, Stage::Domain, correlated())
    }

    /// Shape of a request message, sent or received; pids are opaque, DSP does not type them.
    pub fn request_message() -> Validator<TransferProtocolFields> {
        Validator::new()
            .ensure_not_empty("consumerPid", |f: &TransferProtocolFields| {
                f.consumer_pid.as_deref()
            })
            .ensure_urn("agreementId", |f: &TransferProtocolFields| {
                f.agreement_id.as_deref()
            })
            .ensure_not_empty("format", |f: &TransferProtocolFields| f.format.as_deref())
    }

    /// Shape of a start, completion, suspension or termination message: both pids.
    pub fn pids_message() -> Validator<TransferProtocolFields> {
        Validator::new()
            .ensure_not_empty("consumerPid", |f: &TransferProtocolFields| {
                f.consumer_pid.as_deref()
            })
            .ensure_not_empty("providerPid", |f: &TransferProtocolFields| {
                f.provider_pid.as_deref()
            })
    }

    /// Runs a message-shape validator on the fields extracted from an inbound message.
    fn on_fields(fields: Validator<TransferProtocolFields>) -> Validator<TransferDspTyped> {
        Validator::new().rule(move |typed: &TransferDspTyped| fields.check(&typed.fields))
    }

    /// The body's `@type` must be the message the route handles.
    fn route_matches_body() -> Validator<TransferDspTyped> {
        Validator::new().ensure(
            "@type",
            "message type does not match the endpoint",
            |typed: &TransferDspTyped| &typed.message == typed.route(),
        )
    }

    fn transition(facts: &TransferDspDomain) -> Result<(), Violations> {
        TransferRules::state_transition(
            TransferRules::current_state(&facts.process).as_ref(),
            &facts.typed.message,
            "state",
        )
    }

    fn role(facts: &TransferDspDomain) -> Result<(), Violations> {
        TransferRules::role_for_message(&facts.role, &facts.typed.message, "role")
    }

    fn resume_only(facts: &TransferDspDomain) -> Result<(), Violations> {
        TransferRules::start_received_in(
            &facts.role,
            TransferRules::current_state(&facts.process).as_ref(),
            "state",
        )
    }

    fn semaphore(facts: &TransferDspDomain) -> Result<(), Violations> {
        TransferRules::semaphore(
            &Self::stored_attribute(&facts.process),
            &facts.typed.message,
            &facts.role,
            "stateAttribute",
        )
    }

    fn data_address(facts: &TransferDspDomain) -> Result<(), Violations> {
        TransferRules::data_address_format(
            facts.typed.fields.data_address.is_some(),
            &facts.role,
            "dataAddress",
        )
    }

    fn path_matches_pid(facts: &TransferDspDomain) -> Result<(), Violations> {
        let fields = &facts.typed.fields;
        match facts.typed.path_id() {
            Some(id) => TransferRules::uri_and_pid(
                id,
                fields.consumer_pid.as_deref(),
                fields.provider_pid.as_deref(),
                &facts.role,
                "uriId",
            ),
            None => Ok(()),
        }
    }

    fn correlated(facts: &TransferDspDomain) -> Result<(), Violations> {
        let fields = &facts.typed.fields;
        DspRules::correlate_process_pids(
            Self::stored_pids(&facts.process),
            (
                fields.consumer_pid.as_deref(),
                fields.provider_pid.as_deref(),
            ),
            "pids",
        )
    }

    fn stored_attribute(process: &TransferContextProcessSlot) -> TransferDSPStateAttribute {
        match process {
            TransferContextProcessSlot::Existing(p) => p
                .state_metadata()
                .attribute
                .as_deref()
                .and_then(|s| TransferDSPStateAttribute::from_str(s).ok())
                .unwrap_or(TransferDSPStateAttribute::OnRequest),
            TransferContextProcessSlot::New { .. } => TransferDSPStateAttribute::OnRequest,
        }
    }

    fn stored_pids(process: &TransferContextProcessSlot) -> (Option<&str>, Option<&str>) {
        match process {
            TransferContextProcessSlot::Existing(p) => (
                p.correlation().consumer_pid.as_deref(),
                p.correlation().provider_pid.as_deref(),
            ),
            TransferContextProcessSlot::New { consumer_pid } => (Some(consumer_pid.as_str()), None),
        }
    }
}
