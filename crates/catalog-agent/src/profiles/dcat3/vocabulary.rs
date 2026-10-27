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

//! The DCAT 3 profile: its terms (with `dcterms:` as the recommendation uses), its few rules
//! and the SHACL shapes made from them, as W3C publishes none.

use std::borrow::Cow;

use common::rdf::shacl::ShapeSource;

use crate::entities::jsonld::{Coercion, TermDef, TermTable};
use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;
use crate::profiles::shared::dcat::DcatProfileSpec;
use crate::profiles::shared::rules::{ClassRules, Expect, PropertyRule, RuleSet};

/// DCAT 3 as a DCAT-family spec.
pub struct Dcat3;

impl Dcat3 {
    const PREFIXES: [(&'static str, &'static str); 12] = [
        ("dcat", Vocab::DCAT),
        ("dcterms", Vocab::DCT),
        ("foaf", Vocab::FOAF),
        ("odrl", Vocab::ODRL),
        ("xsd", Vocab::XSD),
        ("rdf", Vocab::RDF),
        ("rdfs", Vocab::RDFS),
        ("vcard", Vocab::VCARD),
        ("skos", Vocab::SKOS),
        ("prov", Vocab::PROV),
        ("adms", Vocab::ADMS),
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

    const TERMS: [TermDef; 36] = [
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
        TermDef::new("spatial", Vocab::DCT_SPATIAL, Coercion::Id, true),
        TermDef::new("temporal", Vocab::DCT_TEMPORAL, Coercion::None, true),
        TermDef::new(
            "accrualPeriodicity",
            Vocab::DCT_ACCRUAL_PERIODICITY,
            Coercion::Id,
            false,
        ),
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

    /// DCAT 3 requires nothing; it only constrains what a value is when it is given.
    fn rule_set() -> RuleSet {
        let resource = |kind| ClassRules {
            kind,
            properties: vec![
                PropertyRule::recommended(Vocab::DCT_TITLE, Expect::Text),
                PropertyRule::optional(Vocab::DCT_DESCRIPTION, Expect::Text),
                PropertyRule::optional(Vocab::DCT_ISSUED, Expect::Temporal).single(),
                PropertyRule::optional(Vocab::DCT_MODIFIED, Expect::Temporal).single(),
                PropertyRule::optional(Vocab::DCT_PUBLISHER, Expect::Iri).single(),
                PropertyRule::optional(Vocab::DCAT_THEME, Expect::Iri),
                PropertyRule::optional(Vocab::DCT_LICENSE, Expect::Iri).single(),
            ],
        };
        let mut distribution = resource(ResourceKind::Distribution);
        distribution.properties.extend([
            PropertyRule::optional(Vocab::DCAT_ACCESS_URL, Expect::Iri),
            PropertyRule::optional(Vocab::DCAT_DOWNLOAD_URL, Expect::Iri),
            PropertyRule::optional(Vocab::DCAT_MEDIA_TYPE, Expect::Iri).single(),
        ]);
        let mut service = resource(ResourceKind::DataService);
        service.properties.push(PropertyRule::recommended(
            Vocab::DCAT_ENDPOINT_URL,
            Expect::Iri,
        ));
        RuleSet {
            classes: vec![
                resource(ResourceKind::Catalog),
                resource(ResourceKind::Dataset),
                resource(ResourceKind::DatasetSeries),
                distribution,
                service,
                ClassRules {
                    kind: ResourceKind::Agent,
                    properties: vec![PropertyRule::recommended(Vocab::FOAF_NAME, Expect::Text)],
                },
            ],
            language_tags: None,
        }
    }
}

impl DcatProfileSpec for Dcat3 {
    const KEY: &'static str = "dcat3";
    const PROFILE_IRI: &'static str = "https://www.w3.org/TR/vocab-dcat-3/";

    fn terms() -> TermTable {
        let mut table = TermTable::new(&Self::PREFIXES, &Self::CLASSES, &Self::TERMS);
        table.terms.push(TermDef::new(
            "catalog",
            Vocab::DCAT_CATALOG_P,
            Coercion::Id,
            true,
        ));
        table.terms.push(TermDef::new(
            "record",
            Vocab::DCAT_RECORD,
            Coercion::Id,
            true,
        ));
        table
    }

    fn rules() -> Option<RuleSet> {
        Some(Self::rule_set())
    }

    fn shapes() -> Vec<ShapeSource> {
        vec![ShapeSource {
            name: "base",
            description: "the DCAT 3 rules of the doctor, as SHACL",
            default: true,
            turtle: Cow::Owned(Self::rule_set().shapes_turtle(Self::PROFILE_IRI)),
        }]
    }
}
