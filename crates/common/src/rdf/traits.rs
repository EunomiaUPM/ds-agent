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

//! Trait definition for deserializing typed domain structures from expanded RDF nodes.

use ymir::errors::Outcome;

use crate::rdf::node::RdfNode;

/// Deserializes a typed domain model from an expanded RDF node.
pub trait FromRdf: Sized {
    /// Optional `@type` IRI used to automatically locate the root node in `@graph` documents.
    const TYPE_IRI: Option<&'static str> = None;

    /// Constructs an instance from an expanded RDF node.
    fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self>;
}
