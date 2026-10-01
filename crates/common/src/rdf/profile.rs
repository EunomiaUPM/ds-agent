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

//! Protocol-specific RDF profile definition for configuring context loaders and expansions.

use crate::rdf::loader::RdfContextLoader;

/// Trait defining a protocol-specific RDF profile.
pub trait RdfProfile: Send + Sync + 'static {
    /// Configures the context loader with preloaded assets or directories.
    fn configure_loader(loader: &mut RdfContextLoader);

    /// Default JSON-LD expand @context string (defaults to W3C XSD).
    fn default_expand_context() -> Option<&'static str> {
        Some(r#"{"@context": {"xsd": "http://www.w3.org/2001/XMLSchema#"}}"#)
    }
}

/// Generic default RDF profile with standard W3C XSD datatypes prefix.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultProfile;

impl RdfProfile for DefaultProfile {
    fn configure_loader(_loader: &mut RdfContextLoader) {}
}
