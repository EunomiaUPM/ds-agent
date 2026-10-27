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

//! The RDF syntaxes the catalog reads and writes, named by media type.

use ymir::errors::{BadFormat, Errors, Outcome};

/// One RDF serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdfSyntax {
    JsonLd,
    Turtle,
    RdfXml,
    NTriples,
}

impl RdfSyntax {
    pub const ALL: [RdfSyntax; 4] = [Self::JsonLd, Self::Turtle, Self::RdfXml, Self::NTriples];

    pub fn media_type(&self) -> &'static str {
        match self {
            Self::JsonLd => "application/ld+json",
            Self::Turtle => "text/turtle",
            Self::RdfXml => "application/rdf+xml",
            Self::NTriples => "application/n-triples",
        }
    }

    /// The extension of a file in this syntax.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::JsonLd => "jsonld",
            Self::Turtle => "ttl",
            Self::RdfXml => "rdf",
            Self::NTriples => "nt",
        }
    }

    /// The syntax a `Content-Type` names; plain JSON is read as JSON-LD.
    pub fn from_media_type(content_type: &str) -> Outcome<Self> {
        let essence = content_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        match essence.as_str() {
            "application/ld+json" | "application/json" => Ok(Self::JsonLd),
            "text/turtle" | "application/x-turtle" => Ok(Self::Turtle),
            "application/rdf+xml" => Ok(Self::RdfXml),
            "application/n-triples" => Ok(Self::NTriples),
            other => Err(Errors::format(
                BadFormat::Received,
                format!(
                    "{other} is not an accepted RDF syntax; send one of {}",
                    Self::ALL.map(|s| s.media_type()).join(", ")
                ),
                None,
            )),
        }
    }

    /// The syntax a `format` query value names (`jsonld`, `turtle`, `ttl`…).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "jsonld" | "json-ld" | "json" => Some(Self::JsonLd),
            "turtle" | "ttl" => Some(Self::Turtle),
            "rdfxml" | "rdf-xml" | "rdf" | "xml" => Some(Self::RdfXml),
            "ntriples" | "n-triples" | "nt" => Some(Self::NTriples),
            _ => None,
        }
    }
}
