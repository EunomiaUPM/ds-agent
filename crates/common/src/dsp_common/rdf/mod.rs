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

//! Dataspace Protocol 2025-1 RDF profile, canonical contexts, and engine integration.

use crate::rdf::canonical::{RdfCanonicalizer, RdfExpansion};
use crate::rdf::engine::RdfEngine;
use crate::rdf::loader::RdfContextLoader;
use crate::rdf::profile::RdfProfile;

pub const DSP_CONTEXT_URL: &str = "https://w3id.org/dspace/2025/1/context.jsonld";
const DSP_CONTEXT_DOC: &str = include_str!("../../../assets/dspace-2025-1-context.jsonld");

pub const DSP_ODRL_PROFILE_URL: &str = "https://w3id.org/dspace/2025/1/odrl-profile.jsonld";
const DSP_ODRL_PROFILE_DOC: &str =
    include_str!("../../../assets/dspace-2025-1-odrl-profile.jsonld");

/// Dataspace Protocol 2025-1 profile implementation.
#[derive(Clone, Copy, Debug, Default)]
pub struct DspProfile;

impl RdfProfile for DspProfile {
    fn configure_loader(loader: &mut RdfContextLoader) {
        let _ = loader.register_asset(DSP_CONTEXT_URL, DSP_CONTEXT_DOC);
        let _ = loader.register_asset(DSP_ODRL_PROFILE_URL, DSP_ODRL_PROFILE_DOC);
        let _ = loader.preload_dir("crates/common/assets");
    }
}

/// Backward compatibility alias for DSP canonicalizer.
pub type DspCanonicalizer = RdfCanonicalizer;

/// Backward compatibility alias for DSP expansion.
pub type DspExpansion = RdfExpansion;

impl DspProfile {
    /// Creates an RdfEngine pre-configured for Dataspace Protocol 2025-1.
    pub fn engine() -> RdfEngine {
        RdfEngine::with_profile::<Self>()
    }
}
