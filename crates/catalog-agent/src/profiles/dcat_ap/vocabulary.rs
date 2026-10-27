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

//! The DCAT-AP 3.0.1 profile: its terms, the official SEMIC shapes it is checked with, and the
//! rewrites into its controlled vocabularies.

use std::borrow::Cow;

use common::rdf::shacl::ShapeSource;
use serde_json::Value;

use crate::entities::graph::{ResourceGraph, Term};
use crate::entities::jsonld::{Coercion, TermDef, TermTable};
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;
use crate::entities::resource_set::ResourceSet;
use crate::entities::vocab::Vocab;
use crate::profiles::shared::dcat::DcatProfileSpec;

/// DCAT-AP 3.0.1 as a DCAT-family spec.
pub struct DcatAp;

impl DcatAp {
    const PREFIXES: [(&'static str, &'static str); 15] = [
        ("dcat", Vocab::DCAT),
        ("dct", Vocab::DCT),
        ("foaf", Vocab::FOAF),
        ("odrl", Vocab::ODRL),
        ("xsd", Vocab::XSD),
        ("rdf", Vocab::RDF),
        ("rdfs", Vocab::RDFS),
        ("vcard", Vocab::VCARD),
        ("skos", Vocab::SKOS),
        ("adms", Vocab::ADMS),
        ("locn", Vocab::LOCN),
        ("spdx", Vocab::SPDX),
        ("owl", Vocab::OWL),
        ("dcatap", Vocab::DCATAP),
        ("dspace", Vocab::DSPACE),
    ];

    const CLASSES: [(&'static str, &'static str); 9] = [
        ("Catalog", Vocab::DCAT_CATALOG),
        ("Dataset", Vocab::DCAT_DATASET),
        ("DatasetSeries", Vocab::DCAT_DATASET_SERIES),
        ("Distribution", Vocab::DCAT_DISTRIBUTION),
        ("DataService", Vocab::DCAT_DATA_SERVICE),
        ("CatalogRecord", Vocab::DCAT_CATALOG_RECORD),
        ("Agent", Vocab::FOAF_AGENT),
        ("Organization", Vocab::FOAF_ORGANIZATION),
        ("Person", Vocab::FOAF_PERSON),
    ];

    const TERMS: [TermDef; 40] = [
        TermDef::new("title", Vocab::DCT_TITLE, Coercion::Language, false),
        TermDef::new(
            "description",
            Vocab::DCT_DESCRIPTION,
            Coercion::Language,
            false,
        ),
        TermDef::new("keyword", Vocab::DCAT_KEYWORD, Coercion::Language, false),
        TermDef::new("name", Vocab::FOAF_NAME, Coercion::Language, false),
        TermDef::new("theme", Vocab::DCAT_THEME, Coercion::Id, true),
        TermDef::new(
            "themeTaxonomy",
            Vocab::DCAT_THEME_TAXONOMY,
            Coercion::Id,
            true,
        ),
        TermDef::new("issued", Vocab::DCT_ISSUED, Coercion::None, false),
        TermDef::new("modified", Vocab::DCT_MODIFIED, Coercion::None, false),
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
            "accrualPeriodicity",
            Vocab::DCT_ACCRUAL_PERIODICITY,
            Coercion::Id,
            false,
        ),
        TermDef::new("status", Vocab::ADMS_STATUS, Coercion::Id, false),
        TermDef::new("type", Vocab::DCT_TYPE, Coercion::Id, false),
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
        TermDef::new("format", Vocab::DCT_FORMAT, Coercion::Id, false),
        TermDef::new("mediaType", Vocab::DCAT_MEDIA_TYPE, Coercion::Id, false),
        TermDef::new("accessURL", Vocab::DCAT_ACCESS_URL, Coercion::Id, true),
        TermDef::new("downloadURL", Vocab::DCAT_DOWNLOAD_URL, Coercion::Id, true),
        TermDef::new(
            "accessService",
            Vocab::DCAT_ACCESS_SERVICE,
            Coercion::Id,
            true,
        ),
        TermDef::new("inSeries", Vocab::DCAT_IN_SERIES, Coercion::Id, true),
        TermDef::new(
            "primaryTopic",
            Vocab::FOAF_PRIMARY_TOPIC,
            Coercion::Id,
            false,
        ),
        TermDef::new("dataset", Vocab::DCAT_DATASET_P, Coercion::Id, true),
        TermDef::new("service", Vocab::DCAT_SERVICE, Coercion::Id, true),
        TermDef::new(
            "distribution",
            Vocab::DCAT_DISTRIBUTION_P,
            Coercion::Id,
            true,
        ),
        TermDef::new("hasPolicy", Vocab::ODRL_HAS_POLICY, Coercion::Id, true),
    ];

    /// SEMIC's shapes as the Interoperable Europe Testbed groups them (see the assets' SOURCE.md);
    /// the two needing background knowledge only run when asked for.
    const SHAPES: [ShapeSource; 5] = [
        ShapeSource {
            name: "base",
            description: "existence, literal ranges and cardinalities",
            default: true,
            turtle: Cow::Borrowed(include_str!(
                "../../../assets/shacl/dcat-ap/3.0.1/shapes.ttl"
            )),
        },
        ShapeSource {
            name: "recommended",
            description: "recommended properties, as warnings",
            default: true,
            turtle: Cow::Borrowed(include_str!(
                "../../../assets/shacl/dcat-ap/3.0.1/shapes_recommended.ttl"
            )),
        },
        ShapeSource {
            name: "deprecated",
            description: "DCAT-AP 2 properties deprecated in 3, as warnings",
            default: true,
            turtle: Cow::Borrowed(include_str!(
                "../../../assets/shacl/dcat-ap/3.0.1/deprecateduris.ttl"
            )),
        },
        ShapeSource {
            name: "range",
            description: "classes of the objects; needs them typed in the document",
            default: false,
            turtle: Cow::Borrowed(include_str!(
                "../../../assets/shacl/dcat-ap/3.0.1/range.ttl"
            )),
        },
        ShapeSource {
            name: "codelists",
            description: "EU controlled vocabularies; needs the code lists in the document",
            default: false,
            turtle: Cow::Borrowed(include_str!(
                "../../../assets/shacl/dcat-ap/3.0.1/mdr-vocabularies.shape.ttl"
            )),
        },
    ];

    /// ISO 639-1 codes and their EU language authority code.
    const LANGUAGES: [(&'static str, &'static str); 24] = [
        ("en", "ENG"),
        ("es", "SPA"),
        ("fr", "FRA"),
        ("de", "DEU"),
        ("it", "ITA"),
        ("pt", "POR"),
        ("nl", "NLD"),
        ("ca", "CAT"),
        ("eu", "EUS"),
        ("gl", "GLG"),
        ("pl", "POL"),
        ("sv", "SWE"),
        ("fi", "FIN"),
        ("da", "DAN"),
        ("el", "ELL"),
        ("cs", "CES"),
        ("hu", "HUN"),
        ("ro", "RON"),
        ("bg", "BUL"),
        ("hr", "HRV"),
        ("sk", "SLK"),
        ("sl", "SLV"),
        ("et", "EST"),
        ("lt", "LIT"),
    ];

    /// File formats by extension or media type, and their EU file-type authority code.
    const FILE_TYPES: [(&'static str, &'static str); 22] = [
        ("csv", "CSV"),
        ("text/csv", "CSV"),
        ("json", "JSON"),
        ("application/json", "JSON"),
        ("jsonld", "JSON_LD"),
        ("application/ld+json", "JSON_LD"),
        ("xml", "XML"),
        ("application/xml", "XML"),
        ("pdf", "PDF"),
        ("application/pdf", "PDF"),
        ("html", "HTML"),
        ("text/html", "HTML"),
        ("rdf", "RDF"),
        ("application/rdf+xml", "RDF_XML"),
        ("ttl", "RDF_TURTLE"),
        ("text/turtle", "RDF_TURTLE"),
        ("parquet", "PARQUET"),
        ("zip", "ZIP"),
        ("application/zip", "ZIP"),
        ("geojson", "GEOJSON"),
        ("xlsx", "XLSX"),
        ("txt", "TXT"),
    ];

    /// A language literal (`en`, `ENG`) as the NAL concept.
    fn language_iri(code: &str) -> Option<String> {
        let lower = code.to_ascii_lowercase();
        Self::LANGUAGES
            .iter()
            .find(|(iso, nal)| *iso == lower || nal.eq_ignore_ascii_case(code))
            .map(|(_, nal)| format!("{}{nal}", Vocab::NAL_LANGUAGE))
    }

    /// A format literal (`csv`, `text/csv`) as the file-type concept.
    fn file_type_iri(format: &str) -> Option<String> {
        let lower = format.to_ascii_lowercase();
        Self::FILE_TYPES
            .iter()
            .find(|(known, _)| *known == lower)
            .map(|(_, nal)| format!("{}{nal}", Vocab::NAL_FILE_TYPE))
    }

    /// `dct:language` literals become NAL IRIs; unknown ones are kept for the doctor to report.
    fn languages(graph: &mut ResourceGraph) {
        let rewritten: Vec<Value> = graph
            .terms(Vocab::DCT_LANGUAGE)
            .into_iter()
            .map(|term| match term {
                Term::Literal(l) => Self::language_iri(&l.lexical())
                    .map(|iri| ResourceGraph::iri(&iri))
                    .unwrap_or_else(|| ResourceGraph::literal(&l.lexical(), None)),
                Term::Iri(iri) => ResourceGraph::iri(iri),
                Term::Node(node) => Value::Object(node.clone()),
            })
            .collect();
        graph.put(Vocab::DCT_LANGUAGE, rewritten);
    }

    /// `dct:format` as a file type: known literals map to the NAL, the rest (DSP transfer
    /// formats such as `http+pull`) are not file formats and stay out of this profile.
    fn formats(graph: &mut ResourceGraph) {
        let rewritten: Vec<Value> = graph
            .terms(Vocab::DCT_FORMAT)
            .into_iter()
            .filter_map(|term| match term {
                Term::Iri(iri) if iri.starts_with(Vocab::NAL_FILE_TYPE) => {
                    Some(ResourceGraph::iri(iri))
                }
                Term::Literal(l) => {
                    Self::file_type_iri(&l.lexical()).map(|iri| ResourceGraph::iri(&iri))
                }
                _ => None,
            })
            .collect();
        graph.put(Vocab::DCT_FORMAT, rewritten);
    }

    /// A `dcat:mediaType` literal (`text/csv`) as its IANA IRI.
    fn media_types(graph: &mut ResourceGraph) {
        let rewritten: Vec<Value> = graph
            .terms(Vocab::DCAT_MEDIA_TYPE)
            .into_iter()
            .map(|term| match term {
                Term::Literal(l) if l.lexical().contains('/') => {
                    ResourceGraph::iri(&format!("{}{}", Vocab::IANA_MEDIA_TYPES, l.lexical()))
                }
                Term::Literal(l) => ResourceGraph::literal(&l.lexical(), None),
                Term::Iri(iri) => ResourceGraph::iri(iri),
                Term::Node(node) => Value::Object(node.clone()),
            })
            .collect();
        graph.put(Vocab::DCAT_MEDIA_TYPE, rewritten);
    }

    /// A distribution reached through a data service is accessed at the service's endpoint.
    fn access_url(resource: &CatalogResource, set: &ResourceSet, graph: &mut ResourceGraph) {
        if graph.has(Vocab::DCAT_ACCESS_URL) {
            return;
        }
        let endpoints: Vec<Value> = resource
            .description()
            .iris(Vocab::DCAT_ACCESS_SERVICE)
            .into_iter()
            .filter_map(|service| set.get(service))
            .filter_map(|service| service.description().first_iri(Vocab::DCAT_ENDPOINT_URL))
            .map(ResourceGraph::iri)
            .collect();
        graph.put(Vocab::DCAT_ACCESS_URL, endpoints);
    }
}

impl DcatProfileSpec for DcatAp {
    const KEY: &'static str = "dcat-ap";
    const PROFILE_IRI: &'static str = "https://semiceu.github.io/DCAT-AP/releases/3.0.1/";

    fn terms() -> TermTable {
        let mut table = TermTable::new(&Self::PREFIXES, &Self::CLASSES, &Self::TERMS);
        table.terms.extend([
            TermDef::new("catalog", Vocab::DCAT_CATALOG_P, Coercion::Id, true),
            TermDef::new("record", Vocab::DCAT_RECORD, Coercion::Id, true),
            TermDef::new("byteSize", Vocab::DCAT_BYTE_SIZE, Coercion::None, false),
            TermDef::new(
                "availability",
                "http://data.europa.eu/r5r/availability",
                Coercion::Id,
                false,
            ),
            TermDef::new(
                "participantId",
                Vocab::DSPACE_PARTICIPANT_ID,
                Coercion::Id,
                false,
            ),
        ]);
        table
    }

    fn shapes() -> Vec<ShapeSource> {
        Self::SHAPES.to_vec()
    }

    fn transform(resource: &CatalogResource, set: &ResourceSet, graph: &mut ResourceGraph) {
        Self::languages(graph);
        if resource.kind() == ResourceKind::Distribution {
            Self::formats(graph);
            Self::media_types(graph);
            Self::access_url(resource, set, graph);
        }
    }
}
