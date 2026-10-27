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

//! What a check found about a resource: a profile's rules, a SHACL shape or an import plan.

use common::rdf::shacl::{ShaclResult, ShaclSeverity};
use serde::Serialize;

/// How bad a finding is: a violation breaks conformance, the rest do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Violation,
    Warning,
    Info,
}

/// One thing a check found about one resource.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub severity: Severity,
    pub resource: String,
    /// The property concerned, as an IRI, when there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    pub message: String,
}

impl From<ShaclSeverity> for Severity {
    fn from(severity: ShaclSeverity) -> Self {
        match severity {
            ShaclSeverity::Violation => Self::Violation,
            ShaclSeverity::Warning => Self::Warning,
            ShaclSeverity::Info => Self::Info,
        }
    }
}

impl From<&ShaclResult> for Finding {
    fn from(result: &ShaclResult) -> Self {
        Self {
            severity: result.severity.into(),
            resource: result.focus_node.clone(),
            property: result.path.clone(),
            message: result.message.clone(),
        }
    }
}
