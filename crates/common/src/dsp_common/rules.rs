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

//! Canonical protocol validation rules defined in Dataspace Protocol 2025-1.
//! Reusable across catalog, negotiation, and transfer agents.

use std::str::FromStr;
use urn::Urn;

use crate::validation::violation::{codes, violation, Path, Violations};

pub struct DspRules;

impl DspRules {
    /// Ensure a string value is a valid URN according to DSP 9.2.
    pub fn is_urn(val: &str, path: impl Into<Path>) -> Result<(), Violations> {
        let p = path.into();
        if val.starts_with("urn:") && Urn::from_str(val).is_ok() {
            Ok(())
        } else {
            Err(violation(p, codes::MALFORMED, "must be a valid URN"))
        }
    }

    /// Ensure an optional string is present and conforms to URN syntax.
    pub fn is_urn_opt(val: Option<&str>, path: impl Into<Path>) -> Result<(), Violations> {
        let p = path.into();
        match val {
            Some(s) => Self::is_urn(s, p),
            None => Err(violation(p, codes::MISSING, "field is required")),
        }
    }

    /// Correlate path parameter PID with message body PID as defined in DSP 9.2.
    pub fn correlate_pids(
        uri_pid: &str,
        body_pid: &str,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        let uri_urn = Urn::from_str(uri_pid);
        let body_urn = Urn::from_str(body_pid);

        match (uri_urn, body_urn) {
            (Ok(u1), Ok(u2)) if u1.to_string() == u2.to_string() => Ok(()),
            _ if uri_pid == body_pid => Ok(()),
            _ => Err(violation(
                p,
                codes::NOT_ALLOWED,
                "URI PID and body PID do not match",
            )),
        }
    }

    /// Verify that a JSON-LD payload references canonical DSP context.
    pub fn valid_dsp_context(
        doc: &serde_json::Value,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        let ctx = doc.get("@context");

        let has_dsp_ctx = match ctx {
            Some(serde_json::Value::String(s)) => s.contains("dspace"),
            Some(serde_json::Value::Array(arr)) => arr.iter().any(|v| match v {
                serde_json::Value::String(s) => s.contains("dspace"),
                _ => false,
            }),
            _ => false,
        };

        if has_dsp_ctx {
            Ok(())
        } else {
            Err(violation(
                p,
                codes::MALFORMED,
                "missing or invalid DSP @context",
            ))
        }
    }

    /// Verify that the JSON-LD @type matches the expected DSP message type.
    pub fn expected_type(
        doc: &serde_json::Value,
        expected: &str,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        let type_val = doc.get("@type");

        let matches = match type_val {
            Some(serde_json::Value::String(s)) => {
                s == expected || s.ends_with(&format!(":{expected}")) || s.ends_with(&format!("/{expected}"))
            }
            Some(serde_json::Value::Array(arr)) => arr.iter().any(|v| match v {
                serde_json::Value::String(s) => {
                    s == expected || s.ends_with(&format!(":{expected}")) || s.ends_with(&format!("/{expected}"))
                }
                _ => false,
            }),
            _ => false,
        };

        if matches {
            Ok(())
        } else {
            Err(violation(
                p,
                codes::NOT_ALLOWED,
                format!("expected message @type '{expected}'"),
            ))
        }
    }
}
