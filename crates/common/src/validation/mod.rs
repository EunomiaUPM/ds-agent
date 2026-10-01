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

//! Protocol-neutral validation: rules, stages, and the failures they report.
//! Rendering a failure onto a wire error belongs to whichever protocol answers.
//!
//! A [`Rule`] is one pure, synchronous check over a subject; any closure or function with the
//! right shape is one. A [`Validator`] runs rules in stages: a stage collects every failure,
//! and the next stage runs only if the previous one passed. Failures come back as
//! [`Violations`], each with a [`Path`] into the subject, a [`ViolationCode`] and a message. A
//! [`ValidatorRegistry`] picks the validator by message type.
//!
//! ## 1. Writing rules
//!
//! A rule returns `Ok(())` or the violations it found. [`violation()`] builds the common case of
//! one failure. The shared codes are in [`codes`]; agents number their own from 1000.
//!
//! ```rust,ignore
//! use common::validation::{codes, violation, Violations};
//!
//! fn pid_present(m: &Msg) -> Result<(), Violations> {
//!     m.pid
//!         .as_ref()
//!         .map(|_| ())
//!         .ok_or_else(|| violation("consumerPid", codes::MISSING, "is required"))
//! }
//! ```
//!
//! Reusable rule catalogs live next to their domain: `crate::auth::AuthRules` and
//! `crate::dsp_common::DspRules`.
//!
//! ## 2. Building a validator
//!
//! Rules in the same stage all run, so the caller sees every problem at once. `then()` opens a
//! stage that only runs on a clean subject, so a format rule never complains about a field an
//! earlier rule already found missing.
//!
//! ```rust,ignore
//! use common::validation::Validator;
//!
//! let validator = Validator::new()
//!     .rule(pid_present)
//!     .ensure_not_empty("format", |m: &Msg| m.format.as_deref())
//!     .then()
//!     .ensure_urn("consumerPid", |m: &Msg| m.pid.as_deref())
//!     .when(|m: &Msg| m.is_push(), callback_required);
//!
//! validator.validate(&message)?;
//! ```
//!
//! `ensure` turns any predicate into a rule. A validator is itself a rule, and `merge` adds
//! another validator's stages to this one.
//!
//! ## 3. Nested fields and lists
//!
//! `field`, `field_opt` and `each` run a validator on a part of the subject and prefix the
//! paths, so failures read like the document.
//!
//! ```rust,ignore
//! let order = Validator::<Order>::new()
//!     .field("address", |o| &o.address, address_validator())
//!     .each("items", |o| &o.items, item_validator());
//!
//! // A failure in the third item reads "items[2].sku: must be a valid URN".
//! ```
//!
//! ## 4. One validator per message type
//!
//! A [`ValidatorRegistry`] maps a key to its validators. Registering the same key twice adds
//! rules instead of replacing them, so a profile can extend the core set. An unregistered key
//! fails, so forgetting to register is caught by tests rather than letting messages through.
//!
//! ```rust,ignore
//! use common::validation::ValidatorRegistry;
//!
//! let mut reg = ValidatorRegistry::new();
//! reg.register(TransferDSPMessageType::TransferRequestMessage, Self::request_dsp());
//! reg.register(TransferDSPMessageType::TransferStartMessage, Self::start_dsp());
//!
//! reg.validate(&message_type, &context)?;
//! ```
//!
//! ## 5. Reporting
//!
//! [`Violations`] keeps the order the rules ran in, so `code()` (the first failure) is the most
//! fundamental one. `to_reasons()` gives `"<path>: <message>"` strings for a wire error and
//! `Display` joins them with `; `. Turning them into an HTTP or DSP error is up to the caller:
//!
//! ```rust,ignore
//! AuthValidators::tenant_id_validator()
//!     .validate(&raw.to_string())
//!     .map_err(|vs| Errors::format(BadFormat::Received, vs.to_string(), None))?;
//! ```
//!
//! A violation can echo the offending value with `with_value`, but only when it cannot be a
//! credential: the value goes back to the peer inside the error.

pub mod rule;
pub mod validator;
pub mod violation;

pub use rule::Rule;
pub use validator::{Validator, ValidatorRegistry};
pub use violation::{codes, violation, Path, Violation, ViolationCode, Violations};

#[cfg(test)]
mod tests;
