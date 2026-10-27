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

//! What an imported document becomes: the catalog resources it describes, with their ids,
//! parents and descriptions, and what keeps any of them from being stored. It reads the
//! document only; what the catalog already holds is the import service's to check.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use common::rdf::RdfGraph;
use oxrdf::{NamedOrBlankNode, NamedOrBlankNodeRef};
use serde_json::{Map, Value};
use urn::Urn;

use crate::entities::findings::{Finding, Severity};
use crate::entities::graph::ResourceGraph;
use crate::entities::resource_kind::{Parent, ResourceKind};
use crate::entities::vocab::Vocab;

/// One resource of the document, as it will be stored.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedResource {
    /// Its IRI (or `_:` label) in the document.
    pub source: String,
    pub id: Urn,
    pub kind: ResourceKind,
    pub parent: Option<Urn>,
    pub description: ResourceGraph,
}

/// The resources a document describes, parents first, and the problems found planning them.
#[derive(Debug, Clone, Default)]
pub struct ImportPlan {
    pub resources: Vec<PlannedResource>,
    pub issues: Vec<Finding>,
}

/// A resource found in the document, before its description is read.
struct Found {
    subject: NamedOrBlankNode,
    source: String,
    kind: ResourceKind,
    id: Urn,
}

impl ImportPlan {
    /// Links the catalog keeps as parents, not in descriptions.
    const MEMBERSHIPS: [&'static str; 5] = [
        Vocab::DCAT_DATASET_P,
        Vocab::DCAT_SERVICE,
        Vocab::DCAT_RECORD,
        Vocab::DCAT_CATALOG_P,
        Vocab::DCAT_DISTRIBUTION_P,
    ];

    /// The order resources are written in, so a parent is stored before what hangs from it.
    const WRITE_ORDER: [ResourceKind; 7] = [
        ResourceKind::Catalog,
        ResourceKind::Agent,
        ResourceKind::DataService,
        ResourceKind::DatasetSeries,
        ResourceKind::Dataset,
        ResourceKind::CatalogRecord,
        ResourceKind::Distribution,
    ];

    /// Plans `graph`. With `target`, its catalogs are not stored and what they list goes there;
    /// without, a member no catalog lists goes to the document's first catalog, else `main`.
    pub fn build(graph: &RdfGraph, target: Option<&Urn>, main: &Urn) -> Self {
        let mut plan = Self::default();
        let found = plan.classify(graph);
        // A catalog merged into the target stands for it when its members are placed.
        let mut catalogs: HashMap<String, Urn> = HashMap::new();
        for f in found.iter().filter(|f| f.kind == ResourceKind::Catalog) {
            catalogs.insert(
                f.source.clone(),
                target.cloned().unwrap_or_else(|| f.id.clone()),
            );
        }
        let fallback = target
            .cloned()
            .or_else(|| {
                found
                    .iter()
                    .find(|f| f.kind == ResourceKind::Catalog)
                    .map(|f| f.id.clone())
            })
            .unwrap_or_else(|| main.clone());
        let stored: Vec<&Found> = found
            .iter()
            .filter(|f| !(target.is_some() && f.kind == ResourceKind::Catalog))
            .collect();
        let ids: HashMap<String, Urn> = stored
            .iter()
            .map(|f| (f.source.clone(), f.id.clone()))
            .collect();
        let datasets: HashMap<String, Urn> = stored
            .iter()
            .filter(|f| f.kind == ResourceKind::Dataset)
            .map(|f| (f.source.clone(), f.id.clone()))
            .collect();
        let resources: HashSet<NamedOrBlankNode> =
            found.iter().map(|f| f.subject.clone()).collect();
        let inline = |node: NamedOrBlankNodeRef<'_>| !resources.contains(&node.into_owned());

        for f in &stored {
            let parent = match f.kind.parent() {
                Parent::None => None,
                Parent::Catalog => Some(
                    plan.parent_in(graph, f, &catalogs)
                        .unwrap_or_else(|| fallback.clone()),
                ),
                Parent::Dataset => match plan.parent_in(graph, f, &datasets) {
                    Some(dataset) => Some(dataset),
                    None => {
                        plan.issue(
                            Severity::Violation,
                            &f.source,
                            Vocab::DCAT_DISTRIBUTION_P,
                            "no dataset of the document lists this distribution",
                        );
                        continue;
                    }
                },
            };
            let mut node = graph.node(f.subject.as_ref(), &inline);
            node.remove("@id");
            for membership in Self::MEMBERSHIPS {
                node.remove(membership);
            }
            Self::rewrite(&mut node, &ids);
            let Ok(mut description) = ResourceGraph::from_node(Value::Object(node)) else {
                continue;
            };
            if !f.source.starts_with("urn:")
                && !f.source.starts_with("_:")
                && !description.has(Vocab::DCT_IDENTIFIER)
            {
                description.put(
                    Vocab::DCT_IDENTIFIER,
                    vec![ResourceGraph::literal(&f.source, None)],
                );
            }
            if let Some((term, reason)) = f.kind.missing_essential(&description) {
                plan.issue(Severity::Violation, &f.source, term, reason);
            }
            plan.resources.push(PlannedResource {
                source: f.source.clone(),
                id: f.id.clone(),
                kind: f.kind,
                parent,
                description,
            });
        }
        plan.resources.sort_by_key(|r| {
            Self::WRITE_ORDER
                .iter()
                .position(|k| *k == r.kind)
                .unwrap_or(usize::MAX)
        });
        plan
    }

    /// Whether nothing found keeps the plan from being written.
    pub fn is_clean(&self) -> bool {
        self.issues
            .iter()
            .all(|i| i.severity != Severity::Violation)
    }

    pub fn issue(
        &mut self,
        severity: Severity,
        resource: &str,
        property: &str,
        message: impl Into<String>,
    ) {
        self.issues.push(Finding {
            severity,
            resource: resource.to_string(),
            property: Some(property.to_string()),
            message: message.into(),
        });
    }

    /// The subjects typed as catalog resources, each with the id it is stored under.
    fn classify(&mut self, graph: &RdfGraph) -> Vec<Found> {
        let mut found = Vec::new();
        for subject in graph.subjects() {
            let types = graph.types(subject.as_ref());
            let Some(kind) = ResourceKind::from_classes(types.iter().map(String::as_str)) else {
                continue;
            };
            let (source, id) = match subject {
                NamedOrBlankNode::NamedNode(n) => {
                    let iri = n.as_str();
                    let id = match Urn::from_str(iri) {
                        Ok(urn) if iri.starts_with("urn:") => urn,
                        _ => kind.mint_for(iri),
                    };
                    (iri.to_string(), id)
                }
                NamedOrBlankNode::BlankNode(b) => (format!("_:{}", b.as_str()), kind.mint()),
            };
            found.push(Found {
                subject: subject.clone(),
                source,
                kind,
                id,
            });
        }
        found
    }

    /// The parent among `parents` that lists `found` through its membership link; the first
    /// when several do, with a warning.
    fn parent_in(
        &mut self,
        graph: &RdfGraph,
        found: &Found,
        parents: &HashMap<String, Urn>,
    ) -> Option<Urn> {
        let membership = found.kind.membership()?;
        let mut distinct: Vec<Urn> = Vec::new();
        for subject in graph.subjects_linking(membership, found.subject.as_ref()) {
            if let Some(parent) = parents.get(&Self::source_of(&subject)) {
                if !distinct.contains(parent) {
                    distinct.push(parent.clone());
                }
            }
        }
        if distinct.len() > 1 {
            let severity = if found.kind == ResourceKind::Distribution {
                Severity::Violation
            } else {
                Severity::Warning
            };
            self.issue(
                severity,
                &found.source,
                membership,
                format!("listed by {} parents; it keeps the first", distinct.len()),
            );
        }
        distinct.into_iter().next()
    }

    fn source_of(subject: &NamedOrBlankNode) -> String {
        match subject {
            NamedOrBlankNode::NamedNode(n) => n.as_str().to_string(),
            NamedOrBlankNode::BlankNode(b) => format!("_:{}", b.as_str()),
        }
    }

    /// References to the document's resources, at any depth, name the ids they are stored under.
    fn rewrite(node: &mut Map<String, Value>, ids: &HashMap<String, Urn>) {
        for (key, value) in node.iter_mut() {
            if key == "@id" {
                continue;
            }
            Self::rewrite_value(value, ids);
        }
    }

    fn rewrite_value(value: &mut Value, ids: &HashMap<String, Urn>) {
        match value {
            Value::Array(values) => values.iter_mut().for_each(|v| Self::rewrite_value(v, ids)),
            Value::Object(object) if object.contains_key("@value") => {}
            Value::Object(object) => {
                if let Some(id) = object
                    .get("@id")
                    .and_then(Value::as_str)
                    .and_then(|id| ids.get(id))
                {
                    object.insert("@id".into(), Value::String(id.to_string()));
                }
                Self::rewrite(object, ids);
            }
            _ => {}
        }
    }
}
