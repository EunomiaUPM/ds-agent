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

//! A projection ready to serve: the document, its media type and what the preview left out.

use common::rdf::RdfSyntax;
use serde_json::Value;

use crate::entities::resource_set::Truncation;

/// The document of a projection: JSON-LD as a value, other syntaxes as text.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionBody {
    JsonLd(Value),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub root: String,
    pub profile: String,
    pub profile_iri: String,
    pub syntax: RdfSyntax,
    pub truncation: Truncation,
    pub body: ProjectionBody,
}

impl Projection {
    /// The file name a download takes, e.g. `catalog-1234-dcat-ap.ttl`.
    pub fn file_name(&self) -> String {
        let root: String = self
            .root
            .trim_start_matches("urn:")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        format!("{root}-{}.{}", self.profile, self.syntax.extension())
    }
}
