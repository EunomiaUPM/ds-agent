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

//! Eunomia: the JSON-LD context of the agents' own RPC messages, typed like the DSP messages
//! they become and reusing the DSP vocabulary wherever the meaning is the same.

pub mod document;

use std::sync::LazyLock;

use crate::dsp_common::odrl::ODRL_CONTEXT_URL;
use crate::rdf::engine::RdfEngine;
use crate::rdf::loader::RdfContextLoader;
use crate::rdf::profile::RdfProfile;

pub const EUNOMIA_CONTEXT_URL: &str = "https://w3id.org/eunomia/v1/context.jsonld";
/// Context of the catalog agent's management API: DCAT resources in our own terms.
pub const EUNOMIA_CATALOG_CONTEXT_URL: &str = "https://w3id.org/eunomia/v1/catalog-context.jsonld";
/// Context of the connector API: templates and instances.
pub const EUNOMIA_CONNECTOR_CONTEXT_URL: &str =
    "https://w3id.org/eunomia/v1/connector-context.jsonld";
/// Context of the keystore API: parameters, secrets and the application config.
pub const EUNOMIA_KEYSTORE_CONTEXT_URL: &str = "https://w3id.org/eunomia/v1/keystore-context.jsonld";
/// Namespace every eunomia term expands into.
pub const EUNOMIA_NS: &str = "https://w3id.org/eunomia/v1/";
const EUNOMIA_CONTEXT_DOC: &str = include_str!("../../assets/eunomia-v1-context.jsonld");
const EUNOMIA_CATALOG_CONTEXT_DOC: &str =
    include_str!("../../assets/eunomia-catalog-v1-context.jsonld");
const EUNOMIA_CONNECTOR_CONTEXT_DOC: &str =
    include_str!("../../assets/eunomia-connector-v1-context.jsonld");
const EUNOMIA_KEYSTORE_CONTEXT_DOC: &str =
    include_str!("../../assets/eunomia-keystore-v1-context.jsonld");
const ODRL_CONTEXT_DOC: &str = include_str!("../../assets/odrl-context.jsonld");

static EUNOMIA_ENGINE: LazyLock<RdfEngine> =
    LazyLock::new(RdfEngine::with_profile::<EunomiaProfile>);

/// RDF profile of eunomia messages and catalog resources; the contexts are embedded, so they are
/// never fetched.
#[derive(Clone, Copy, Debug, Default)]
pub struct EunomiaProfile;

impl RdfProfile for EunomiaProfile {
    fn configure_loader(loader: &mut RdfContextLoader) {
        loader
            .register_asset(EUNOMIA_CONTEXT_URL, EUNOMIA_CONTEXT_DOC)
            .expect("embedded eunomia context is valid JSON-LD");
        loader
            .register_asset(EUNOMIA_CATALOG_CONTEXT_URL, EUNOMIA_CATALOG_CONTEXT_DOC)
            .expect("embedded eunomia catalog context is valid JSON-LD");
        loader
            .register_asset(EUNOMIA_CONNECTOR_CONTEXT_URL, EUNOMIA_CONNECTOR_CONTEXT_DOC)
            .expect("embedded eunomia connector context is valid JSON-LD");
        loader
            .register_asset(EUNOMIA_KEYSTORE_CONTEXT_URL, EUNOMIA_KEYSTORE_CONTEXT_DOC)
            .expect("embedded eunomia keystore context is valid JSON-LD");
        loader
            .register_asset(ODRL_CONTEXT_URL, ODRL_CONTEXT_DOC)
            .expect("embedded ODRL context is valid JSON-LD");
    }
}

impl EunomiaProfile {
    /// Process-wide eunomia engine.
    pub fn shared() -> &'static RdfEngine {
        &EUNOMIA_ENGINE
    }

    /// The full IRI a eunomia term expands to.
    pub fn term(term: &str) -> String {
        format!("{EUNOMIA_NS}{term}")
    }
}

#[cfg(test)]
mod tests;
