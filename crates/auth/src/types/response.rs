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

//! What to do after a GNAP answer.

/// Authority's answer: issue over OID4VCI, present over OID4VP, or wait.
pub enum VcWhatResponse {
    Issuance(String),
    Presentation(String),
    Wait,
}

use serde::Serialize;
use ymir::types::gnap::grant_request::access::ResourceAccess;
use ymir::types::verification::PresentationRequest;

/// Peer's answer: done with what it granted, present over OID4VP, or wait.
pub enum TokenWhatResponse {
    Completed(Vec<ResourceAccess>),
    Presentation(String),
    Wait,
}

pub enum RotationOutcome {
    Rotated,
    Refused,
}

/// Who asks for a presentation, as far as it could be checked against the peer of the grant.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum VerifierCheck {
    /// Signed by the DID of the peer.
    Verified { did: String },
    /// Signed by another DID: never presented to.
    Mismatch { did: String },
    /// Unsigned, but answered to the host the grant was sent to.
    SameOrigin,
    Unsigned,
    /// Its signature is not valid: never presented to.
    Untrusted { reason: String },
}

impl VerifierCheck {
    pub fn blocks(&self) -> bool {
        matches!(self, Self::Mismatch { .. } | Self::Untrusted { .. })
    }
}

/// A pending presentation request, resolved without answering it.
#[derive(Serialize, Debug)]
pub struct ResolvedPresentation {
    pub request: Option<PresentationRequest>,
    pub verifier: VerifierCheck,
}
