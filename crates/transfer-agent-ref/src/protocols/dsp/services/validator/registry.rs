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

//! Composable validator registries for inbound DSP messages, outbound RPC commands, and edge validation.

use common::dsp_common::DspRules;
use common::validation::{Validator, ValidatorRegistry, Violations, codes, violation};
use std::str::FromStr;

use crate::protocols::dsp::entities::context_common::TransferContextProcessSlot;
use crate::protocols::dsp::entities::context_dsp::{
    TransferDSPContextDomain, TransferDSPContextTyped,
};
use crate::protocols::dsp::entities::context_rpc::TransferRPCContextDomain;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;
use crate::protocols::dsp::services::validator::rules::TransferRules;

pub struct TransferValidators;

impl TransferValidators {
    /// Inbound DSP validator registry checking structural syntax and domain invariants.
    pub fn dsp_registry() -> ValidatorRegistry<TransferDSPMessageType, TransferDSPContextDomain> {
        let mut reg = ValidatorRegistry::new();
        reg.register(
            TransferDSPMessageType::TransferRequestMessage,
            Self::request_dsp(),
        );
        reg.register(
            TransferDSPMessageType::TransferStartMessage,
            Self::start_dsp(),
        );
        reg.register(
            TransferDSPMessageType::TransferCompletionMessage,
            Self::completion_dsp(),
        );
        reg.register(
            TransferDSPMessageType::TransferSuspensionMessage,
            Self::suspension_dsp(),
        );
        reg.register(
            TransferDSPMessageType::TransferTerminationMessage,
            Self::termination_dsp(),
        );
        reg
    }

    /// Outbound RPC validator registry checking symmetrical invariants before sending to peers.
    pub fn rpc_registry() -> ValidatorRegistry<TransferDSPMessageType, TransferRPCContextDomain> {
        let mut reg = ValidatorRegistry::new();
        reg.register(
            TransferDSPMessageType::TransferRequestMessage,
            Self::request_rpc(),
        );
        reg.register(
            TransferDSPMessageType::TransferStartMessage,
            Self::start_rpc(),
        );
        reg.register(
            TransferDSPMessageType::TransferCompletionMessage,
            Self::completion_rpc(),
        );
        reg.register(
            TransferDSPMessageType::TransferSuspensionMessage,
            Self::suspension_rpc(),
        );
        reg.register(
            TransferDSPMessageType::TransferTerminationMessage,
            Self::termination_rpc(),
        );
        reg
    }

    /// Edge validator registry checking wire format and presence prior to database lookups.
    pub fn edge_registry() -> ValidatorRegistry<TransferDSPMessageType, TransferDSPContextTyped> {
        let mut reg = ValidatorRegistry::new();
        reg.register(
            TransferDSPMessageType::TransferRequestMessage,
            Self::request_edge(),
        );
        reg.register(
            TransferDSPMessageType::TransferStartMessage,
            Self::start_edge(),
        );
        reg.register(
            TransferDSPMessageType::TransferCompletionMessage,
            Self::completion_edge(),
        );
        reg.register(
            TransferDSPMessageType::TransferSuspensionMessage,
            Self::suspension_edge(),
        );
        reg.register(
            TransferDSPMessageType::TransferTerminationMessage,
            Self::termination_edge(),
        );
        reg
    }

    fn request_edge() -> Validator<TransferDSPContextTyped> {
        Validator::new().rule(|ctx: &TransferDSPContextTyped| {
            let mut v = Violations::new();
            if let Some(consumer_pid) = &ctx.fields.consumer_pid {
                if let Err(vs) = DspRules::is_urn(consumer_pid, "consumerPid") {
                    v.extend(vs);
                }
            } else {
                v.push(common::validation::Violation::new(
                    "consumerPid",
                    codes::MISSING,
                    "is required",
                ));
            }
            if let Some(agreement_id) = &ctx.fields.agreement_id {
                if let Err(vs) = DspRules::is_urn(agreement_id, "agreementId") {
                    v.extend(vs);
                }
            } else {
                v.push(common::validation::Violation::new(
                    "agreementId",
                    codes::MISSING,
                    "is required",
                ));
            }
            if ctx.fields.format.as_deref().unwrap_or("").trim().is_empty() {
                v.push(common::validation::Violation::new(
                    "format",
                    codes::MISSING,
                    "is required",
                ));
            }
            v.into_result()
        })
    }

    fn start_edge() -> Validator<TransferDSPContextTyped> {
        Validator::new().rule(|ctx: &TransferDSPContextTyped| {
            let mut v = Violations::new();
            Self::check_urn_opt(&mut v, "consumerPid", ctx.fields.consumer_pid.as_deref());
            Self::check_urn_opt(&mut v, "providerPid", ctx.fields.provider_pid.as_deref());
            v.into_result()
        })
    }

    fn completion_edge() -> Validator<TransferDSPContextTyped> {
        Validator::new().rule(|ctx: &TransferDSPContextTyped| {
            let mut v = Violations::new();
            Self::check_urn_opt(&mut v, "consumerPid", ctx.fields.consumer_pid.as_deref());
            Self::check_urn_opt(&mut v, "providerPid", ctx.fields.provider_pid.as_deref());
            v.into_result()
        })
    }

    fn suspension_edge() -> Validator<TransferDSPContextTyped> {
        Validator::new().rule(|ctx: &TransferDSPContextTyped| {
            let mut v = Violations::new();
            Self::check_urn_opt(&mut v, "consumerPid", ctx.fields.consumer_pid.as_deref());
            Self::check_urn_opt(&mut v, "providerPid", ctx.fields.provider_pid.as_deref());
            if ctx.fields.code.as_deref().unwrap_or("").trim().is_empty() {
                v.push(common::validation::Violation::new(
                    "code",
                    codes::MISSING,
                    "is required",
                ));
            }
            v.into_result()
        })
    }

    fn termination_edge() -> Validator<TransferDSPContextTyped> {
        Validator::new().rule(|ctx: &TransferDSPContextTyped| {
            let mut v = Violations::new();
            Self::check_urn_opt(&mut v, "consumerPid", ctx.fields.consumer_pid.as_deref());
            Self::check_urn_opt(&mut v, "providerPid", ctx.fields.provider_pid.as_deref());
            if ctx.fields.code.as_deref().unwrap_or("").trim().is_empty() {
                v.push(common::validation::Violation::new(
                    "code",
                    codes::MISSING,
                    "is required",
                ));
            }
            v.into_result()
        })
    }

    fn request_dsp() -> Validator<TransferDSPContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(
                    &mut v,
                    "consumerPid",
                    ctx.typed.fields.consumer_pid.as_deref(),
                );
                Self::check_urn_opt(
                    &mut v,
                    "agreementId",
                    ctx.typed.fields.agreement_id.as_deref(),
                );
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                if let Err(vs) =
                    TransferRules::role_for_message(&ctx.role, &ctx.typed.message, "role")
                {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn start_dsp() -> Validator<TransferDSPContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(
                    &mut v,
                    "consumerPid",
                    ctx.typed.fields.consumer_pid.as_deref(),
                );
                Self::check_urn_opt(
                    &mut v,
                    "providerPid",
                    ctx.typed.fields.provider_pid.as_deref(),
                );
                if let Some(uri_id) = ctx.typed.rdf.parsed.raw.path_id.as_deref() {
                    Self::check_urn_opt(&mut v, "uriId", Some(uri_id));
                }
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                let attr = Self::extract_attribute(&ctx.process);

                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                if let Err(vs) =
                    TransferRules::role_for_message(&ctx.role, &ctx.typed.message, "role")
                {
                    v.extend(vs);
                }
                if let Err(vs) =
                    TransferRules::semaphore(&attr, &ctx.typed.message, &ctx.role, "stateAttribute")
                {
                    v.extend(vs);
                }
                if let Err(vs) = TransferRules::data_address_format(
                    ctx.typed.fields.data_address.is_some(),
                    &ctx.role,
                    &attr,
                    "dataAddress",
                ) {
                    v.extend(vs);
                }
                if let Some(uri_id) = ctx.typed.rdf.parsed.raw.path_id.as_deref() {
                    if let Err(vs) = TransferRules::uri_and_pid(
                        uri_id,
                        ctx.typed.fields.consumer_pid.as_deref(),
                        ctx.typed.fields.provider_pid.as_deref(),
                        &ctx.role,
                        "uriId",
                    ) {
                        v.extend(vs);
                    }
                }
                let (proc_cons, proc_prov) = Self::extract_stored_pids(&ctx.process);
                if let Err(vs) = TransferRules::correlation(
                    proc_cons,
                    proc_prov,
                    ctx.typed.fields.consumer_pid.as_deref(),
                    ctx.typed.fields.provider_pid.as_deref(),
                    "pids",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn completion_dsp() -> Validator<TransferDSPContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(
                    &mut v,
                    "consumerPid",
                    ctx.typed.fields.consumer_pid.as_deref(),
                );
                Self::check_urn_opt(
                    &mut v,
                    "providerPid",
                    ctx.typed.fields.provider_pid.as_deref(),
                );
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                let (proc_cons, proc_prov) = Self::extract_stored_pids(&ctx.process);
                if let Err(vs) = TransferRules::correlation(
                    proc_cons,
                    proc_prov,
                    ctx.typed.fields.consumer_pid.as_deref(),
                    ctx.typed.fields.provider_pid.as_deref(),
                    "pids",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn suspension_dsp() -> Validator<TransferDSPContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(
                    &mut v,
                    "consumerPid",
                    ctx.typed.fields.consumer_pid.as_deref(),
                );
                Self::check_urn_opt(
                    &mut v,
                    "providerPid",
                    ctx.typed.fields.provider_pid.as_deref(),
                );
                if ctx
                    .typed
                    .fields
                    .code
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    v.push(common::validation::Violation::new(
                        "code",
                        codes::MISSING,
                        "is required",
                    ));
                }
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                let (proc_cons, proc_prov) = Self::extract_stored_pids(&ctx.process);
                if let Err(vs) = TransferRules::correlation(
                    proc_cons,
                    proc_prov,
                    ctx.typed.fields.consumer_pid.as_deref(),
                    ctx.typed.fields.provider_pid.as_deref(),
                    "pids",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn termination_dsp() -> Validator<TransferDSPContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(
                    &mut v,
                    "consumerPid",
                    ctx.typed.fields.consumer_pid.as_deref(),
                );
                Self::check_urn_opt(
                    &mut v,
                    "providerPid",
                    ctx.typed.fields.provider_pid.as_deref(),
                );
                if ctx
                    .typed
                    .fields
                    .code
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    v.push(common::validation::Violation::new(
                        "code",
                        codes::MISSING,
                        "is required",
                    ));
                }
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferDSPContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                let (proc_cons, proc_prov) = Self::extract_stored_pids(&ctx.process);
                if let Err(vs) = TransferRules::correlation(
                    proc_cons,
                    proc_prov,
                    ctx.typed.fields.consumer_pid.as_deref(),
                    ctx.typed.fields.provider_pid.as_deref(),
                    "pids",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn request_rpc() -> Validator<TransferRPCContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(&mut v, "consumerPid", ctx.typed.consumer_pid.as_deref());
                Self::check_urn_opt(&mut v, "agreementId", ctx.typed.agreement_id.as_deref());
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn start_rpc() -> Validator<TransferRPCContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(&mut v, "consumerPid", ctx.typed.consumer_pid.as_deref());
                Self::check_urn_opt(&mut v, "providerPid", ctx.typed.provider_pid.as_deref());
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn completion_rpc() -> Validator<TransferRPCContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(&mut v, "consumerPid", ctx.typed.consumer_pid.as_deref());
                Self::check_urn_opt(&mut v, "providerPid", ctx.typed.provider_pid.as_deref());
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn suspension_rpc() -> Validator<TransferRPCContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(&mut v, "consumerPid", ctx.typed.consumer_pid.as_deref());
                Self::check_urn_opt(&mut v, "providerPid", ctx.typed.provider_pid.as_deref());
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn termination_rpc() -> Validator<TransferRPCContextDomain> {
        Validator::new()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                Self::check_urn_opt(&mut v, "consumerPid", ctx.typed.consumer_pid.as_deref());
                Self::check_urn_opt(&mut v, "providerPid", ctx.typed.provider_pid.as_deref());
                v.into_result()
            })
            .then()
            .rule(|ctx: &TransferRPCContextDomain| {
                let mut v = Violations::new();
                let current_state = TransferRules::current_state(&ctx.process);
                if let Err(vs) = TransferRules::state_transition(
                    current_state.as_ref(),
                    &ctx.typed.message,
                    "state",
                ) {
                    v.extend(vs);
                }
                v.into_result()
            })
    }

    fn check_urn_opt(v: &mut Violations, field: &'static str, val: Option<&str>) {
        match val {
            Some(s) if s.starts_with("urn:") => {}
            Some(_) => v.push(common::validation::Violation::new(
                field,
                codes::MALFORMED,
                "must be a valid URN",
            )),
            None => v.push(common::validation::Violation::new(
                field,
                codes::MISSING,
                "is required",
            )),
        }
    }

    fn extract_attribute(process: &TransferContextProcessSlot) -> TransferDSPStateAttribute {
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

    fn extract_stored_pids(process: &TransferContextProcessSlot) -> (Option<&str>, Option<&str>) {
        match process {
            TransferContextProcessSlot::Existing(p) => (
                p.correlation().consumer_pid.as_deref(),
                p.correlation().provider_pid.as_deref(),
            ),
            TransferContextProcessSlot::New { consumer_pid } => (Some(consumer_pid.as_str()), None),
        }
    }
}
