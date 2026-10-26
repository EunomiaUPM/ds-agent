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

//! The DSP projection (DSP 6): the DSP terms where the DSP context defines them (`participantId`,
//! `dataset`, `distribution`, `service`, `hasPolicy`, `format`, `accessService`, `endpointURL`),
//! every other property as a `dct:` / `dcat:` compact IRI, so the output stays RDF.

use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::{OdrlOffer, OdrlPolicyInfo, OdrlTypes};
use serde_json::{json, Map, Value};
use ymir::errors::Outcome;

use crate::entities::graph::{ResourceGraph, Term};
use crate::entities::jsonld::{Compactor, TermTable};
use crate::entities::odrl_offer::OdrlOffer as StoredOffer;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::resource_set::ResourceSet;
use crate::entities::vocab::Vocab;
use crate::profiles::dsp::rules::DspRules;
use crate::profiles::shared::effective::EffectiveGraph;
use crate::profiles::shared::rules::RuleSet;
use crate::profiles::{Finding, ProfileAdapterTrait, ProfileDiagnosis, Severity};

/// DSP 2025-1 as a profile.
pub struct DspProfileAdapter {
    /// The prefixes the DSP context declares, and its class terms.
    terms: TermTable,
    rules: RuleSet,
    schemas: DspRules,
}

impl DspProfileAdapter {
    pub const KEY: &'static str = "dsp";
    pub const PROFILE_IRI: &'static str = "https://w3id.org/dspace/2025/1/";

    /// Predicates the DSP terms carry, never repeated as compact IRIs.
    const OWN: [&'static str; 6] = [
        Vocab::DSPACE_PARTICIPANT_ID,
        Vocab::DCT_FORMAT,
        Vocab::DCAT_ACCESS_SERVICE,
        Vocab::DCAT_ENDPOINT_URL,
        Vocab::DCAT_ENDPOINT_DESCRIPTION,
        Vocab::ODRL_HAS_POLICY,
    ];

    pub fn new() -> Self {
        Self {
            terms: TermTable::new(
                &[
                    ("dct", Vocab::DCT),
                    ("dcat", Vocab::DCAT),
                    ("odrl", Vocab::ODRL),
                    ("dspace", Vocab::DSPACE),
                    ("xsd", Vocab::XSD),
                ],
                &[
                    ("Catalog", Vocab::DCAT_CATALOG),
                    ("Dataset", Vocab::DCAT_DATASET),
                    ("Distribution", Vocab::DCAT_DISTRIBUTION),
                    ("DataService", Vocab::DCAT_DATA_SERVICE),
                ],
                &[],
            ),
            rules: DspRules::rule_set(),
            schemas: DspRules::new(),
        }
    }

    /// The main catalog, with the other catalogs it shows, as a peer receives it.
    pub fn catalog_document(&self, set: &ResourceSet, catalog: &CatalogResource) -> Value {
        let mut node = self.catalog(set, catalog);
        node.insert("@context".into(), Self::context_field());
        Value::Object(node)
    }

    /// A dataset with its offers and distributions, as a peer receives it.
    pub fn dataset_document(&self, set: &ResourceSet, dataset: &CatalogResource) -> Value {
        let mut node = self.dataset(set, dataset);
        node.insert("@context".into(), Self::context_field());
        Value::Object(node)
    }

    fn catalog(&self, set: &ResourceSet, catalog: &CatalogResource) -> Map<String, Value> {
        let id = catalog.id().to_string();
        let mut node = self.properties(catalog, ResourceKind::Catalog);
        if let Some(participant) = catalog.description().first_value(Vocab::DSPACE_PARTICIPANT_ID) {
            node.insert("participantId".into(), json!(participant));
        }
        if catalog.is_main() {
            let subs: Vec<Value> = set
                .sub_catalogs(&id)
                .into_iter()
                .map(|sub| Value::Object(self.catalog(set, sub)))
                .collect();
            Self::insert_non_empty(&mut node, "catalog", subs);
        }
        let datasets: Vec<Value> = set
            .children_of(&id, ResourceKind::Dataset)
            .into_iter()
            .map(|dataset| Value::Object(self.dataset(set, dataset)))
            .collect();
        Self::insert_non_empty(&mut node, "dataset", datasets);
        let services: Vec<Value> = set
            .children_of(&id, ResourceKind::DataService)
            .into_iter()
            .map(|service| Value::Object(Self::service(service)))
            .collect();
        Self::insert_non_empty(&mut node, "service", services);
        node
    }

    fn dataset(&self, set: &ResourceSet, dataset: &CatalogResource) -> Map<String, Value> {
        let id = dataset.id().to_string();
        let mut node = self.properties(dataset, ResourceKind::Dataset);
        let offers: Vec<Value> = set.offers_of(&id).iter().filter_map(Self::offer).collect();
        node.insert("hasPolicy".into(), Value::Array(offers));
        let distributions: Vec<Value> = set
            .children_of(&id, ResourceKind::Distribution)
            .into_iter()
            .map(|distribution| Value::Object(self.distribution(set, distribution)))
            .collect();
        node.insert("distribution".into(), Value::Array(distributions));
        node
    }

    fn distribution(&self, set: &ResourceSet, distribution: &CatalogResource) -> Map<String, Value> {
        let mut node = self.properties(distribution, ResourceKind::Distribution);
        let description = distribution.description();
        if let Some(format) = description.first_value(Vocab::DCT_FORMAT) {
            node.insert("format".into(), json!(format));
        }
        if let Some(service) = description.first_iri(Vocab::DCAT_ACCESS_SERVICE) {
            let access = match set.get(service) {
                Some(found) => Value::Object(Self::service(found)),
                None => json!(service),
            };
            node.insert("accessService".into(), access);
        }
        node
    }

    /// A data service as DSP lists it: its id, type and endpoint.
    fn service(service: &CatalogResource) -> Map<String, Value> {
        let description = service.description();
        let mut node = Map::new();
        node.insert("@id".into(), json!(service.id().to_string()));
        node.insert("@type".into(), json!(ResourceKind::DataService.term()));
        if let Some(url) = description.first_value(Vocab::DCAT_ENDPOINT_URL) {
            node.insert("endpointURL".into(), json!(url));
        }
        if let Some(endpoint) = description.first_value(Vocab::DCAT_ENDPOINT_DESCRIPTION) {
            node.insert("endpointDescription".into(), json!(endpoint));
        }
        node
    }

    /// Every property the DSP terms do not carry, as compact IRIs over the DSP prefixes.
    fn properties(&self, resource: &CatalogResource, kind: ResourceKind) -> Map<String, Value> {
        let mut description = resource.description().clone();
        for own in Self::OWN {
            description.remove(own);
        }
        let mut node = Compactor::new(&self.terms).node(description.as_map());
        node.insert("@type".into(), json!(kind.term()));
        node
    }

    /// A stored offer as an ODRL `Offer`, without the empty rule lists DSP schemas reject.
    fn offer(stored: &StoredOffer) -> Option<Value> {
        let info: OdrlPolicyInfo = serde_json::from_value(stored.odrl_offer.clone()).ok()?;
        let offer = OdrlOffer {
            id: stored.id.clone(),
            profile: None,
            permission: info.permission,
            obligation: info.obligation,
            _type: OdrlTypes::Offer,
            prohibition: info.prohibition,
            target: None,
            description: stored.description.clone(),
        }
        .without_empty_rules();
        serde_json::to_value(offer).ok()
    }

    fn insert_non_empty(node: &mut Map<String, Value>, key: &str, values: Vec<Value>) {
        if !values.is_empty() {
            node.insert(key.into(), Value::Array(values));
        }
    }

    fn context_field() -> Value {
        serde_json::to_value(ContextField::default()).unwrap_or_default()
    }

    /// The DSP root catalog carries the provider's identity (DSP 6).
    fn participant_finding(catalog: &CatalogResource) -> Option<Finding> {
        let graph: &ResourceGraph = catalog.description();
        let has_participant = graph
            .terms(Vocab::DSPACE_PARTICIPANT_ID)
            .iter()
            .any(|t| matches!(t, Term::Iri(_) | Term::Literal(_)));
        (catalog.is_main() && !has_participant).then(|| Finding {
            severity: Severity::Violation,
            resource: catalog.id().to_string(),
            property: Some(Vocab::DSPACE_PARTICIPANT_ID.to_string()),
            message: "the root catalog needs participantId".to_string(),
        })
    }

    /// Offers belong to datasets in DSP, never to a catalog.
    fn catalog_offer_finding(set: &ResourceSet, catalog: &CatalogResource) -> Option<Finding> {
        let id = catalog.id().to_string();
        (!set.offers_of(&id).is_empty()).then(|| Finding {
            severity: Severity::Violation,
            resource: id,
            property: Some(Vocab::ODRL_HAS_POLICY.to_string()),
            message: "a DSP catalog carries no hasPolicy; offers go on its datasets".to_string(),
        })
    }
}

impl Default for DspProfileAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileAdapterTrait for DspProfileAdapter {
    fn key(&self) -> &'static str {
        Self::KEY
    }

    fn profile_iri(&self) -> &'static str {
        Self::PROFILE_IRI
    }

    fn context(&self) -> Value {
        json!({ "@context": Self::context_field() })
    }

    fn project(&self, set: &ResourceSet, root: &CatalogResource) -> Outcome<Value> {
        Ok(match root.kind() {
            ResourceKind::Catalog => self.catalog_document(set, root),
            ResourceKind::Dataset => self.dataset_document(set, root),
            ResourceKind::Distribution => {
                let mut node = self.distribution(set, root);
                node.insert("@context".into(), Self::context_field());
                Value::Object(node)
            }
            ResourceKind::DataService => {
                let mut node = self.properties(root, ResourceKind::DataService);
                node.extend(Self::service(root));
                node.insert("@context".into(), Self::context_field());
                Value::Object(node)
            }
            other => {
                let mut node = Compactor::new(&self.terms).node(root.description().as_map());
                node.insert("@type".into(), json!(self.terms.class_term(other.class_iri())));
                node.insert("@context".into(), Self::context_field());
                Value::Object(node)
            }
        })
    }

    fn diagnose(&self, set: &ResourceSet, root: &CatalogResource, deep: bool) -> Outcome<ProfileDiagnosis> {
        let checked: Vec<&CatalogResource> = if deep {
            std::iter::once(root)
                .chain(set.iter().filter(|r| r.id() != root.id()))
                .collect()
        } else {
            vec![root]
        };
        let mut findings = Vec::new();
        for resource in checked {
            findings.extend(self.rules.check(resource.kind(), &EffectiveGraph::of(resource, set)));
            if resource.kind() == ResourceKind::Catalog {
                findings.extend(Self::participant_finding(resource));
                findings.extend(Self::catalog_offer_finding(set, resource));
            }
        }
        // The schema covers what the peer receives: the whole projection of the root.
        let projection = self.project(set, root)?;
        let schema_kind = match root.kind() {
            ResourceKind::Catalog if root.is_main() => ResourceKind::Catalog,
            ResourceKind::Dataset => ResourceKind::Dataset,
            _ => ResourceKind::Agent,
        };
        findings.extend(self.schemas.check_schema(schema_kind, root.id().as_ref(), &projection));
        Ok(ProfileDiagnosis::from_findings(self, findings))
    }
}
