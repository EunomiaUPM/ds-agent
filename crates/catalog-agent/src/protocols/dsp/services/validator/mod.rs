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

//! What a catalog RPC body must carry before a peer is asked.

use urn::Urn;
use ymir::errors::{ErrorReason, Errors, Outcome};

use crate::protocols::dsp::entities::message_types::CatalogDspMessageType;
use crate::protocols::dsp::entities::protocol_fields::CatalogProtocolFields;

/// The checks of the catalog RPC.
pub struct CatalogRpcRules;

impl CatalogRpcRules {
    /// The peer always; the dataset, as a URN, when one is asked for.
    pub fn check(message: CatalogDspMessageType, fields: &CatalogProtocolFields) -> Outcome<()> {
        let mut reasons = Vec::new();
        if fields.associated_agent_peer.as_deref().is_none_or(str::is_empty) {
            reasons.push(Self::reason("associatedAgentPeer", "is required"));
        }
        if message == CatalogDspMessageType::DatasetRequestMessage {
            match fields.dataset.as_deref() {
                None => reasons.push(Self::reason("dataset", "is required")),
                Some(id) if id.parse::<Urn>().is_err() => {
                    reasons.push(Self::reason("dataset", "must be a URN"))
                }
                _ => {}
            }
        }
        if reasons.is_empty() {
            Ok(())
        } else {
            Err(Errors::validation_with(reasons))
        }
    }

    fn reason(path: &str, message: &str) -> ErrorReason {
        ErrorReason {
            path: path.to_string(),
            code: 1,
            message: message.to_string(),
        }
    }
}
