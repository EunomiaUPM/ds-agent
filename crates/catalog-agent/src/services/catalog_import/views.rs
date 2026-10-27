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

//! What an import answers: the SHACL report, the problems of the plan and what was (or would
//! be) written.

use common::rdf::shacl::ShaclReport;
use serde::Serialize;

use crate::entities::findings::Finding;
use crate::entities::import_plan::PlannedResource;

/// Whether a resource is new or replaces a stored one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportAction {
    Create,
    Update,
}

/// One resource of the document and where it went.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedResource {
    /// Its IRI in the document.
    pub source: String,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub action: ImportAction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub profile: String,
    pub dry_run: bool,
    /// Valid against the shapes and with nothing in the plan keeping it out.
    pub conforms: bool,
    /// Whether it was written.
    pub imported: bool,
    pub validation: ShaclReport,
    pub issues: Vec<Finding>,
    pub created: usize,
    pub updated: usize,
    pub resources: Vec<ImportedResource>,
}

impl ImportedResource {
    pub fn assemble(planned: &PlannedResource, action: ImportAction) -> Self {
        Self {
            source: planned.source.clone(),
            id: planned.id.to_string(),
            kind: planned.kind.term().to_string(),
            parent: planned.parent.as_ref().map(ToString::to_string),
            action,
        }
    }
}
