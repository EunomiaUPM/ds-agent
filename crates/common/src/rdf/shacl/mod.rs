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

//! SHACL validation with rudof's SHACL Core engine: shapes compiled once per group, run over an
//! [`RdfGraph`] and read back as a report. Imports in a shapes graph are never fetched.

pub mod report;

use std::borrow::Cow;
use std::sync::Arc;

use rudof_rdf::rdf_core::RDFFormat;
use rudof_rdf::rdf_impl::{OxigraphInMemory, ReaderMode};
use shacl::ir::IRSchema;
use shacl::validator::processor::{GraphValidation, ShaclProcessor};
use shacl::validator::store::Graph;
use shacl::validator::{ShaclConfig, ShaclValidationMode};
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::rdf::graph::RdfGraph;

pub use report::{ShaclReport, ShaclResult, ShaclSeverity};

/// One shapes graph as Turtle, under the name a caller picks it by.
#[derive(Debug, Clone)]
pub struct ShapeSource {
    pub name: &'static str,
    pub description: &'static str,
    /// Whether it runs when the caller names no groups.
    pub default: bool,
    pub turtle: Cow<'static, str>,
}

/// A compiled shapes graph.
pub struct ShapeGroup {
    pub name: &'static str,
    pub description: &'static str,
    pub default: bool,
    schema: IRSchema,
}

/// The shape groups of one profile, and the prefixes its messages compact IRIs with.
#[derive(Clone)]
pub struct ShapeSet {
    groups: Arc<Vec<ShapeGroup>>,
    prefixes: &'static [(&'static str, &'static str)],
}

impl ShapeSet {
    /// Compiles every source; a shapes graph that does not parse is a bug of the caller.
    pub fn compile(
        sources: Vec<ShapeSource>,
        prefixes: &'static [(&'static str, &'static str)],
    ) -> Outcome<Self> {
        let groups = sources
            .into_iter()
            .map(|source| {
                let schema = IRSchema::from_str(
                    &source.turtle,
                    &RDFFormat::Turtle,
                    None,
                    &ReaderMode::default(),
                )
                .map_err(|e| Errors::crazy(format!("shapes {}: {e}", source.name), None))?;
                Ok(ShapeGroup {
                    name: source.name,
                    description: source.description,
                    default: source.default,
                    schema,
                })
            })
            .collect::<Outcome<Vec<_>>>()?;
        Ok(Self {
            groups: Arc::new(groups),
            prefixes,
        })
    }

    pub fn groups(&self) -> impl Iterator<Item = &ShapeGroup> {
        self.groups.iter()
    }

    /// The groups that run when none are named.
    pub fn defaults(&self) -> Vec<&'static str> {
        self.groups
            .iter()
            .filter(|g| g.default)
            .map(|g| g.name)
            .collect()
    }

    /// Validates `graph` against the named groups, or the default ones when `checks` is empty.
    pub fn validate(&self, graph: &RdfGraph, checks: &[String]) -> Outcome<ShaclReport> {
        let selected = self.select(checks)?;
        let mut results = Vec::new();
        for group in &selected {
            let mut validation = GraphValidation::new(Graph::from(Self::store(graph)));
            let report = validation
                .validate(
                    &group.schema,
                    &ShaclValidationMode::Native,
                    &ShaclConfig::new(),
                )
                .map_err(|e| {
                    Errors::crazy(format!("SHACL validation ({}): {e}", group.name), None)
                })?;
            results.extend(
                report
                    .results()
                    .iter()
                    .map(|r| ShaclResult::read(group.name, r, self.prefixes)),
            );
        }
        Ok(ShaclReport::new(
            selected.iter().map(|g| g.name.to_string()).collect(),
            results,
        ))
    }

    fn select(&self, checks: &[String]) -> Outcome<Vec<&ShapeGroup>> {
        if checks.is_empty() {
            return Ok(self.groups.iter().filter(|g| g.default).collect());
        }
        checks
            .iter()
            .map(|name| {
                self.groups.iter().find(|g| g.name == name).ok_or_else(|| {
                    Errors::format(
                        BadFormat::Received,
                        format!(
                            "unknown check {name}; available: {}",
                            self.groups
                                .iter()
                                .map(|g| g.name)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        None,
                    )
                })
            })
            .collect()
    }

    fn store(graph: &RdfGraph) -> OxigraphInMemory {
        let mut store = OxigraphInMemory::new();
        for triple in graph.triples() {
            // Adding a triple to the in-memory store cannot fail.
            let _ = store.add_triple_ref(triple.subject, triple.predicate, triple.object);
        }
        store
    }
}
