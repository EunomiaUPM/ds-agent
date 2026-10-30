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

//! A resource as the management API reads and writes it: compact JSON-LD in the eunomia catalog
//! context. A body is read through RDF, so any context and any aliasing a caller uses is fine.

use std::str::FromStr;

use common::eunomia::{EunomiaProfile, EUNOMIA_CATALOG_CONTEXT_URL};
use common::rdf::RdfCanonicalizer;
use serde_json::{json, Map, Value};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::graph::ResourceGraph;
use crate::entities::jsonld::{Coercion, Compactor, TermDef, TermTable};
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::{Parent, ResourceKind};
use crate::entities::vocab::Vocab;

/// A resource read from a management API body.
#[derive(Debug, Clone)]
pub struct AdminDocument {
    pub kind: ResourceKind,
    /// The `@id` it names, when it names one.
    pub id: Option<Urn>,
    /// Its catalog or dataset, as `inCatalog` / `ofDataset` say.
    pub parent: Option<Urn>,
    pub description: ResourceGraph,
}

impl AdminDocument {
    /// Expands `body` (the catalog context is assumed when it has none) and reads its one resource.
    pub async fn read(mut body: Value) -> Outcome<Self> {
        if let Value::Object(object) = &mut body {
            object
                .entry("@context")
                .or_insert_with(|| json!(EUNOMIA_CATALOG_CONTEXT_URL));
        }
        let expansion = EunomiaProfile::shared().expand(&body).await?;
        let nodes = expansion.expanded.as_array().cloned().unwrap_or_default();
        let mut roots = nodes.into_iter().filter_map(|node| {
            let kind = Self::kind_of(&node)?;
            Some((kind, node))
        });
        let (kind, node) = roots.next().ok_or_else(|| {
            Self::malformed("the body describes no Catalog, Dataset, DatasetSeries, Distribution, DataService, Agent or CatalogRecord")
        })?;
        if roots.next().is_some() {
            return Err(Self::malformed("the body describes more than one resource"));
        }
        Self::from_node(kind, node)
    }

    /// The resource as compact JSON-LD, with its parent as `inCatalog` / `ofDataset`.
    pub fn render(resource: &CatalogResource) -> Value {
        let mut node = Self::node(resource);
        node.insert("@context".into(), json!(EUNOMIA_CATALOG_CONTEXT_URL));
        Value::Object(node)
    }

    /// Several resources as one `@graph`.
    pub fn render_graph(resources: &[CatalogResource]) -> Value {
        let graph: Vec<Value> = resources
            .iter()
            .map(|r| Value::Object(Self::node(r)))
            .collect();
        json!({ "@context": EUNOMIA_CATALOG_CONTEXT_URL, "@graph": graph })
    }

    /// The terms of the catalog context, as the asset in `common` defines them.
    pub fn terms() -> TermTable {
        TermTable::new(&Vocab::PREFIXES, &Self::CLASSES, &Self::TERMS)
    }

    fn node(resource: &CatalogResource) -> Map<String, Value> {
        let table = Self::terms();
        let mut node = Compactor::new(&table).node(resource.description().as_map());
        if let Some(parent) = resource.parent_id() {
            let term = match resource.kind().parent() {
                Parent::Dataset => "ofDataset",
                _ => "inCatalog",
            };
            node.insert(term.into(), json!(parent.to_string()));
        }
        node
    }

    fn from_node(kind: ResourceKind, node: Value) -> Outcome<Self> {
        let mut node = match node {
            Value::Object(node) => node,
            _ => return Err(Self::malformed("a resource must be a node object")),
        };
        let parent = Self::parent_of(kind, node.remove("@reverse"))?;
        let id = match node.get("@id").and_then(Value::as_str) {
            Some(id) if id.starts_with("_:") => None,
            Some(id) => Some(Urn::from_str(id).map_err(|_| {
                Self::malformed(&format!("resource ids are URNs, {id} is not one"))
            })?),
            None => None,
        };
        node.remove("@id");
        Self::resolve_relative(&mut node)?;
        let mut description = ResourceGraph::from_node(Value::Object(node))?;
        for managed in Vocab::MANAGED {
            description.remove(managed);
        }
        description.reduce_embedded_resources();
        Ok(Self {
            kind,
            id,
            parent,
            description,
        })
    }

    /// The parent the reverse membership link names; other reverse links are not stored.
    fn parent_of(kind: ResourceKind, reverse: Option<Value>) -> Outcome<Option<Urn>> {
        let Some(membership) = kind.membership() else {
            return Ok(None);
        };
        let parents: Vec<&str> = reverse
            .as_ref()
            .and_then(|r| r.get(membership))
            .and_then(Value::as_array)
            .map(|links| {
                links
                    .iter()
                    .filter_map(|l| l.get("@id")?.as_str())
                    .collect()
            })
            .unwrap_or_default();
        match parents.as_slice() {
            [] => Ok(None),
            [one] => Urn::from_str(one)
                .map(Some)
                .map_err(|_| Self::malformed(&format!("{one} is not the URN of a resource"))),
            _ => Err(Self::malformed("a resource belongs to one parent only")),
        }
    }

    /// A bare value under an `@id` term expands below the document base: a language code
    /// (`"en"`) is kept as the literal it is, anything else must have been an absolute IRI.
    fn resolve_relative(node: &mut Map<String, Value>) -> Outcome<()> {
        for (predicate, values) in node.iter_mut().filter(|(k, _)| !k.starts_with('@')) {
            let Some(values) = values.as_array_mut() else {
                continue;
            };
            for value in values {
                let Some(relative) = value
                    .get("@id")
                    .and_then(Value::as_str)
                    .and_then(|id| id.strip_prefix(RdfCanonicalizer::DOCUMENT_BASE))
                    .map(|rest| rest.trim_start_matches('/').to_string())
                else {
                    continue;
                };
                if predicate == Vocab::DCT_LANGUAGE {
                    *value = ResourceGraph::literal(&relative, None);
                } else {
                    return Err(Self::malformed(&format!(
                        "{predicate} takes an absolute IRI, {relative} is not one"
                    )));
                }
            }
        }
        Ok(())
    }

    fn kind_of(node: &Value) -> Option<ResourceKind> {
        node.get("@type")?
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .find_map(ResourceKind::from_class)
    }

    fn malformed(reason: &str) -> Errors {
        Errors::format(BadFormat::Received, reason.to_string(), None)
    }

    const CLASSES: [(&'static str, &'static str); 14] = [
        ("Catalog", Vocab::DCAT_CATALOG),
        ("Dataset", Vocab::DCAT_DATASET),
        ("DatasetSeries", Vocab::DCAT_DATASET_SERIES),
        ("DataService", Vocab::DCAT_DATA_SERVICE),
        ("CatalogRecord", Vocab::DCAT_CATALOG_RECORD),
        ("Distribution", Vocab::DCAT_DISTRIBUTION),
        ("Agent", Vocab::FOAF_AGENT),
        ("Organization", Vocab::FOAF_ORGANIZATION),
        ("Person", Vocab::FOAF_PERSON),
        ("Kind", "http://www.w3.org/2006/vcard/ns#Kind"),
        ("PeriodOfTime", "http://purl.org/dc/terms/PeriodOfTime"),
        ("Location", "http://purl.org/dc/terms/Location"),
        (
            "LicenseDocument",
            "http://purl.org/dc/terms/LicenseDocument",
        ),
        ("Concept", "http://www.w3.org/2004/02/skos/core#Concept"),
    ];

    const TERMS: [TermDef; 51] = [
        TermDef::new("title", Vocab::DCT_TITLE, Coercion::Language, false),
        TermDef::new(
            "description",
            Vocab::DCT_DESCRIPTION,
            Coercion::Language,
            false,
        ),
        TermDef::new("keyword", Vocab::DCAT_KEYWORD, Coercion::Language, false),
        TermDef::new("name", Vocab::FOAF_NAME, Coercion::Language, false),
        TermDef::new(
            "versionNotes",
            "http://www.w3.org/ns/adms#versionNotes",
            Coercion::Language,
            false,
        ),
        TermDef::new("theme", Vocab::DCAT_THEME, Coercion::Id, true),
        TermDef::new(
            "themeTaxonomy",
            Vocab::DCAT_THEME_TAXONOMY,
            Coercion::Id,
            true,
        ),
        TermDef::new(
            "issued",
            Vocab::DCT_ISSUED,
            Coercion::Typed(Vocab::XSD_DATE_TIME),
            false,
        ),
        TermDef::new(
            "modified",
            Vocab::DCT_MODIFIED,
            Coercion::Typed(Vocab::XSD_DATE_TIME),
            false,
        ),
        TermDef::new("identifier", Vocab::DCT_IDENTIFIER, Coercion::None, false),
        TermDef::new("version", Vocab::DCAT_VERSION, Coercion::None, false),
        TermDef::new("conformsTo", Vocab::DCT_CONFORMS_TO, Coercion::Id, true),
        TermDef::new("creator", Vocab::DCT_CREATOR, Coercion::Id, false),
        TermDef::new("publisher", Vocab::DCT_PUBLISHER, Coercion::Id, false),
        TermDef::new(
            "rightsHolder",
            Vocab::DCT_RIGHTS_HOLDER,
            Coercion::Id,
            false,
        ),
        TermDef::new(
            "contactPoint",
            Vocab::DCAT_CONTACT_POINT,
            Coercion::None,
            true,
        ),
        TermDef::new("homepage", Vocab::FOAF_HOMEPAGE, Coercion::Id, false),
        TermDef::new("landingPage", Vocab::DCAT_LANDING_PAGE, Coercion::Id, true),
        TermDef::new("page", "http://xmlns.com/foaf/0.1/page", Coercion::Id, true),
        TermDef::new("language", Vocab::DCT_LANGUAGE, Coercion::Id, true),
        TermDef::new("license", Vocab::DCT_LICENSE, Coercion::Id, false),
        TermDef::new(
            "accessRights",
            Vocab::DCT_ACCESS_RIGHTS,
            Coercion::Id,
            false,
        ),
        TermDef::new("rights", Vocab::DCT_RIGHTS, Coercion::Id, false),
        TermDef::new("spatial", Vocab::DCT_SPATIAL, Coercion::Id, true),
        TermDef::new("temporal", Vocab::DCT_TEMPORAL, Coercion::None, true),
        TermDef::new(
            "startDate",
            "http://www.w3.org/ns/dcat#startDate",
            Coercion::None,
            false,
        ),
        TermDef::new(
            "endDate",
            "http://www.w3.org/ns/dcat#endDate",
            Coercion::None,
            false,
        ),
        TermDef::new(
            "accrualPeriodicity",
            Vocab::DCT_ACCRUAL_PERIODICITY,
            Coercion::Id,
            false,
        ),
        TermDef::new("status", Vocab::ADMS_STATUS, Coercion::Id, false),
        TermDef::new("endpointURL", Vocab::DCAT_ENDPOINT_URL, Coercion::Id, false),
        TermDef::new(
            "endpointDescription",
            Vocab::DCAT_ENDPOINT_DESCRIPTION,
            Coercion::Id,
            false,
        ),
        TermDef::new(
            "servesDataset",
            Vocab::DCAT_SERVES_DATASET,
            Coercion::Id,
            true,
        ),
        TermDef::new("format", Vocab::DCT_FORMAT, Coercion::None, false),
        TermDef::new("mediaType", Vocab::DCAT_MEDIA_TYPE, Coercion::Id, false),
        TermDef::new("accessURL", Vocab::DCAT_ACCESS_URL, Coercion::Id, true),
        TermDef::new("downloadURL", Vocab::DCAT_DOWNLOAD_URL, Coercion::Id, true),
        TermDef::new(
            "accessService",
            Vocab::DCAT_ACCESS_SERVICE,
            Coercion::Id,
            true,
        ),
        TermDef::new(
            "byteSize",
            Vocab::DCAT_BYTE_SIZE,
            Coercion::Typed("http://www.w3.org/2001/XMLSchema#nonNegativeInteger"),
            false,
        ),
        TermDef::new(
            "availability",
            "http://data.europa.eu/r5r/availability",
            Coercion::Id,
            false,
        ),
        TermDef::new("inSeries", Vocab::DCAT_IN_SERIES, Coercion::Id, true),
        TermDef::new(
            "primaryTopic",
            Vocab::FOAF_PRIMARY_TOPIC,
            Coercion::Id,
            false,
        ),
        TermDef::new(
            "participantId",
            Vocab::DSPACE_PARTICIPANT_ID,
            Coercion::Id,
            false,
        ),
        TermDef::new(
            "fn",
            "http://www.w3.org/2006/vcard/ns#fn",
            Coercion::None,
            false,
        ),
        TermDef::new(
            "hasEmail",
            "http://www.w3.org/2006/vcard/ns#hasEmail",
            Coercion::Id,
            false,
        ),
        TermDef::new(
            "label",
            "http://www.w3.org/2000/01/rdf-schema#label",
            Coercion::None,
            false,
        ),
        TermDef::new(
            "seeAlso",
            "http://www.w3.org/2000/01/rdf-schema#seeAlso",
            Coercion::Id,
            true,
        ),
        TermDef::new("dctType", Vocab::DCT_TYPE, Coercion::Id, false),
        TermDef::new(
            "checksum",
            "http://spdx.org/rdf/terms#checksum",
            Coercion::None,
            false,
        ),
        TermDef::new(
            "hasVersion",
            "http://purl.org/dc/terms/hasVersion",
            Coercion::Id,
            true,
        ),
        // The data schema; profiles publish it (or the dataset's) as `dct:conformsTo`.
        TermDef::new("schema", Vocab::EUNOMIA_SCHEMA, Coercion::Id, false),
        // `draft` or `published`; changed by the publish and draft actions, kept by a replace.
        TermDef::new(
            "publicationState",
            Vocab::EUNOMIA_PUBLICATION_STATE,
            Coercion::None,
            false,
        ),
    ];
}
