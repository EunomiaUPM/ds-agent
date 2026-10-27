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

//! The adapter every DCAT-family profile shares. A profile of the family is a spec: its terms,
//! its rules or SHACL shapes and the transformations it applies; the rest is common.

use std::marker::PhantomData;

use common::rdf::shacl::{ShapeSet, ShapeSource};
use common::rdf::RdfGraph;
use serde_json::{json, Value};
use ymir::errors::Outcome;

use crate::entities::graph::ResourceGraph;
use crate::entities::jsonld::{Compactor, TermTable};
use crate::entities::resource::CatalogResource;
use crate::entities::resource_set::ResourceSet;
use crate::entities::vocab::Vocab;
use crate::profiles::shared::effective::EffectiveGraph;
use crate::profiles::shared::rules::RuleSet;
use crate::profiles::{Finding, ProfileAdapterTrait, ProfileDiagnosis};

/// What makes one DCAT-family profile.
pub trait DcatProfileSpec: Send + Sync + 'static {
    const KEY: &'static str;
    const PROFILE_IRI: &'static str;

    /// The terms of the profile's context.
    fn terms() -> TermTable;
    /// What the doctor checks, for a profile with rules of its own; without them it checks the
    /// default shape groups.
    fn rules() -> Option<RuleSet> {
        None
    }
    /// The SHACL shape groups an import is validated with.
    fn shapes() -> Vec<ShapeSource>;
    /// Rewrites the effective graph into the profile's conventions (vocabularies, IRIs).
    fn transform(_resource: &CatalogResource, _set: &ResourceSet, _graph: &mut ResourceGraph) {}
}

/// Projection and diagnosis of a DCAT-family profile.
pub struct DcatAdapter<S: DcatProfileSpec> {
    terms: TermTable,
    rules: Option<RuleSet>,
    shapes: ShapeSet,
    spec: PhantomData<S>,
}

impl<S: DcatProfileSpec> Default for DcatAdapter<S> {
    fn default() -> Self {
        Self {
            terms: S::terms(),
            rules: S::rules(),
            shapes: ShapeSet::compile(S::shapes(), &Vocab::PREFIXES)
                .expect("the embedded shapes compile"),
            spec: PhantomData,
        }
    }
}

impl<S: DcatProfileSpec> DcatAdapter<S> {
    /// The graph a resource is projected and checked as.
    pub fn graph(&self, resource: &CatalogResource, set: &ResourceSet) -> ResourceGraph {
        let mut graph = EffectiveGraph::of(resource, set);
        S::transform(resource, set, &mut graph);
        graph
    }

    /// The SHACL results on the effective graphs, about the checked resources only; a result on
    /// a nested node is the finding of the resource holding it.
    fn shacl_findings(
        &self,
        set: &ResourceSet,
        root: &CatalogResource,
        checked: &[&CatalogResource],
    ) -> Outcome<Vec<Finding>> {
        let graphs: Vec<_> = Self::ordered(set, root)
            .into_iter()
            .map(|r| self.graph(r, set))
            .collect();
        let rdf = RdfGraph::from_nodes(graphs.iter().map(ResourceGraph::as_map))?;
        let ids: Vec<String> = checked.iter().map(|r| r.id().to_string()).collect();
        let report = self
            .shapes
            .validate(&rdf, &[])?
            .attributed(&rdf)
            .about(&ids);
        Ok(report.results.iter().map(Finding::from).collect())
    }

    /// The root first, then the rest of the set, in the order it was loaded.
    fn ordered<'s>(set: &'s ResourceSet, root: &'s CatalogResource) -> Vec<&'s CatalogResource> {
        std::iter::once(root)
            .chain(set.iter().filter(|r| r.id() != root.id()))
            .collect()
    }
}

impl<S: DcatProfileSpec> ProfileAdapterTrait for DcatAdapter<S> {
    fn key(&self) -> &'static str {
        S::KEY
    }

    fn profile_iri(&self) -> &'static str {
        S::PROFILE_IRI
    }

    fn context(&self) -> Value {
        self.terms.context()
    }

    fn project(&self, set: &ResourceSet, root: &CatalogResource) -> Outcome<Value> {
        let compactor = Compactor::new(&self.terms);
        let graph: Vec<Value> = Self::ordered(set, root)
            .into_iter()
            .map(|resource| Value::Object(compactor.node(self.graph(resource, set).as_map())))
            .collect();
        Ok(json!({
            "@context": self.terms.context()["@context"].clone(),
            "@graph": graph,
        }))
    }

    fn diagnose(
        &self,
        set: &ResourceSet,
        root: &CatalogResource,
        deep: bool,
    ) -> Outcome<ProfileDiagnosis> {
        let checked = if deep {
            Self::ordered(set, root)
        } else {
            vec![root]
        };
        let findings = match &self.rules {
            Some(rules) => checked
                .into_iter()
                .flat_map(|resource| rules.check(resource.kind(), &self.graph(resource, set)))
                .collect(),
            None => self.shacl_findings(set, root, &checked)?,
        };
        Ok(ProfileDiagnosis::from_findings(self, findings))
    }

    fn shapes(&self) -> Option<&ShapeSet> {
        Some(&self.shapes)
    }
}
