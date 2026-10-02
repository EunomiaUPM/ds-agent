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

//! Atomic payload checks. The command already carries the extracted fields, so
//! these read primitives and never go back to the message.

use std::str::FromStr;

use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;

/// Format checks on single payload values.
pub struct PayloadValidator;

impl PayloadValidator {
    /// Parse a value as a URN.
    pub fn urn(value: &str) -> Outcome<Urn> {
        Urn::from_str(value).map_err(|e| Errors::parse(format!("value must be a URN: {e}"), None))
    }

    /// The pid the request URI carries (by role) must equal the body's pid — both
    /// parsed as URNs so `urn:uuid:x` and a re-serialised form compare equal.
    pub fn uri_and_pid(
        uri_id: &str,
        consumer_pid: Option<&str>,
        provider_pid: Option<&str>,
        role: &TransferRole,
    ) -> Outcome<()> {
        let body_pid = match role {
            TransferRole::Provider => provider_pid,
            TransferRole::Consumer => consumer_pid,
            TransferRole::Relay => return Err(Errors::parse("Relay has no pid", None)),
        }
        .ok_or_else(|| Errors::parse("body is missing the role's pid", None))?;

        if Self::urn(uri_id)?.to_string() != Self::urn(body_pid)?.to_string() {
            return Err(Errors::parse(
                "URI pid and body pid are not correlated",
                None,
            ));
        }
        Ok(())
    }

    /// The pids in the incoming message must match the stored process's pids.
    pub fn correlation(
        process_consumer: Option<&str>,
        process_provider: Option<&str>,
        message_consumer: Option<&str>,
        message_provider: Option<&str>,
    ) -> Outcome<()> {
        if process_consumer != message_consumer || process_provider != message_provider {
            return Err(Errors::parse(
                "message pids and process pids are not correlated",
                None,
            ));
        }
        Ok(())
    }

    /// A `dataAddress` on a `TransferStart` is only valid from the provider on the
    /// first start (`OnRequest`); a consumer sending one on a later start is invalid.
    pub fn data_address_in_start(
        has_data_address: bool,
        role: &TransferRole,
        attribute: &TransferDSPStateAttribute,
    ) -> Outcome<()> {
        if has_data_address
            && matches!(role, TransferRole::Consumer)
            && !matches!(attribute, TransferDSPStateAttribute::OnRequest)
        {
            return Err(Errors::parse(
                "dataAddress is only allowed in the first provider TransferStart",
                None,
            ));
        }
        Ok(())
    }

    /// JSON-schema validation of the raw message body.
    // TODO: stub until the per-message schemas land.
    pub fn json_schema(_body: &serde_json::Value) -> Outcome<()> {
        Ok(())
    }

    /// Authorization / token check against the peer store.
    // TODO: stub until the peer/token store is available.
    pub fn auth() -> Outcome<()> {
        Ok(())
    }
}
