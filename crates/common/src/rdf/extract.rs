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

//! Turning an expanded message into a protocol's own field set. One impl per DSP
//! protocol: the algorithm is shared, the field set is not.

use ymir::errors::{BadFormat, Errors, Outcome};

use crate::rdf::expanded::{ExpandedDoc, Node};

pub trait ExtractProtocolFields {
    /// The protocol's message-type enum.
    type MessageType: std::fmt::Display;
    /// What this protocol reads off a message.
    type Fields;

    /// The `@type` IRI a message type expands to.
    fn type_iri(message: &Self::MessageType) -> String;

    /// Read the fields. Total by design: which are required depends on the
    /// message type, and that is the validator's call.
    fn extract(node: &Node<'_, '_>) -> Outcome<Self::Fields>;

    /// The message type a node's `@type` names, or `None` if it names none.
    fn message_type(node: &Node<'_, '_>) -> Option<Self::MessageType>;

    /// Bind the message node and the type it declares. Reads the type off the
    /// body: whether it agrees with the route is the validator's call, not this.
    fn root_message<'d, 'a>(
        doc: &'d ExpandedDoc<'a>,
    ) -> Outcome<(Self::MessageType, Node<'d, 'a>)> {
        let mut matching = doc
            .nodes()
            .filter_map(|node| Self::message_type(&node).map(|kind| (kind, node)));
        let found = matching.next().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "expanded body declares no message type",
                None,
            )
        })?;
        if matching.next().is_some() {
            return Err(Errors::format(
                BadFormat::Received,
                "expanded body declares more than one message node",
                None,
            ));
        }
        Ok(found)
    }
}
