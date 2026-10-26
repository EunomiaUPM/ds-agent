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
//! its rules and the transformations it applies; the projection and the diagnosis are common.

use std::marker::PhantomData;

use serde_json::{json, Value};
use ymir::errors::Outcome;

use crate::entities::graph::ResourceGraph;
use crate::entities::jsonld::{Compactor, TermTable};
use crate::entities::resource::CatalogResource;
use crate::entities::resource_set::ResourceSet;
use crate::profiles::shared::effective::EffectiveGraph;
use crate::profiles::shared::rules::RuleSet;
use crate::profiles::{ProfileAdapterTrait, ProfileDiagnosis};

/// What makes one DCAT-family profile.
pub trait DcatProfileSpec: Send + Sync + 'static {
    const KEY: &'static str;
    const PROFILE_IRI: &'static str;

    /// The terms of the profile's context.
    fn terms() -> TermTable;
    /// What the profile requires of each class.
    fn rules() -> RuleSet;
    /// Rewrites the effective graph into the profile's conventions (vocabularies, IRIs).
    fn transform(_resource: &CatalogResource, _set: &ResourceSet, _graph: &mut ResourceGraph) {}
}

/// Projection and diagnosis of a DCAT-family profile.
pub struct DcatAdapter<S: DcatProfileSpec> {
    terms: TermTable,
    rules: RuleSet,
    spec: PhantomData<S>,
}

impl<S: DcatProfileSpec> Default for DcatAdapter<S> {
    fn default() -> Self {
        Self {
            terms: S::terms(),
            rules: S::rules(),
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

    fn diagnose(&self, set: &ResourceSet, root: &CatalogResource, deep: bool) -> Outcome<ProfileDiagnosis> {
        let checked = if deep {
            Self::ordered(set, root)
        } else {
            vec![root]
        };
        let findings = checked
            .into_iter()
            .flat_map(|resource| self.rules.check(resource.kind(), &self.graph(resource, set)))
            .collect();
        Ok(ProfileDiagnosis::from_findings(self, findings))
    }
}
