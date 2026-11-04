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

//! Reads what an ODRL offer asks of a peer. For now only the credential types under `use`:
//! `ovc:credentialSubjectType eq X`, alone or joined by `and`. The rest is left aside as if it were
//! not there, with a warning: nothing is asked for it.

use std::str::FromStr;

use serde_json::Value;
use ymir::types::vcs::VcType;

const CREDENTIAL_TYPE: &str = "ovc:credentialSubjectType";

pub struct PolicyInterpreter;

impl PolicyInterpreter {
    /// The credentials `odrl` asks for to use its target.
    pub fn credentials(odrl: &Value) -> Vec<VcType> {
        let mut found = Vec::new();
        for rule in Self::items(&odrl["permission"]) {
            if Self::term(&rule["action"]).is_some_and(|action| action.ends_with("use")) {
                for constraint in Self::items(&rule["constraint"]) {
                    Self::read(constraint, &mut found);
                }
            }
        }
        found
    }

    /// A constraint and those it joins with `and`.
    fn read(constraint: &Value, found: &mut Vec<VcType>) {
        if let Some(joined) = constraint.get("and") {
            for inner in Self::items(joined) {
                Self::read(inner, found);
            }
            return;
        }
        let left = Self::term(&constraint["leftOperand"]);
        let operator = Self::term(&constraint["operator"]);
        let right = Self::term(&constraint["rightOperand"]);
        match (left, operator, right) {
            (Some(CREDENTIAL_TYPE), Some(op), Some(right)) if op.ends_with("eq") => {
                let credential = Self::credential(right);
                if !found.contains(&credential) {
                    found.push(credential);
                }
            }
            _ => tracing::warn!(%constraint, "constraint left aside: only credential types are read"),
        }
    }

    /// The credential an offer names; `gx:LegalParticipant` is how the templates call it.
    fn credential(name: &str) -> VcType {
        match name {
            "gx:LegalParticipant" => VcType::LegalPerson,
            other => VcType::from_str(other).unwrap_or_else(|never| match never {}),
        }
    }

    /// A value as JSON-LD may write it: alone or as a list.
    fn items(value: &Value) -> Vec<&Value> {
        match value {
            Value::Null => Vec::new(),
            Value::Array(items) => items.iter().collect(),
            single => vec![single],
        }
    }

    /// A term as JSON-LD may write it: plain, or under `@id` or `@value`.
    fn term(value: &Value) -> Option<&str> {
        value
            .as_str()
            .or_else(|| value["@id"].as_str())
            .or_else(|| value["@value"].as_str())
    }
}
