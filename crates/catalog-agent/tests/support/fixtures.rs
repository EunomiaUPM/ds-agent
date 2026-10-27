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

//! The DCAT-AP documents of `tests/fixtures/dcat-ap/`: real catalogs of the EDACCIT deployment.

use std::path::PathBuf;

use common::eunomia::EunomiaProfile;
use common::rdf::{RdfGraph, RdfSyntax};
use serde_json::Value;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dcat-ap")
}

/// The file names, sorted.
pub fn dcat_ap_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir())
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".jsonld"))
        .collect();
    names.sort();
    names
}

pub fn dcat_ap_bytes(name: &str) -> Vec<u8> {
    std::fs::read(dir().join(name)).unwrap()
}

pub fn dcat_ap_json(name: &str) -> Value {
    serde_json::from_slice(&dcat_ap_bytes(name)).unwrap()
}

pub async fn dcat_ap_graph(name: &str) -> RdfGraph {
    RdfGraph::parse(
        RdfSyntax::JsonLd,
        &dcat_ap_bytes(name),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap()
}

/// A JSON-LD document parsed into triples.
pub async fn graph_of(document: &Value) -> RdfGraph {
    RdfGraph::parse(
        RdfSyntax::JsonLd,
        &serde_json::to_vec(document).unwrap(),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap()
}
