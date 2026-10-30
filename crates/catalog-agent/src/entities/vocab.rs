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

//! IRIs of the vocabularies the catalog stores and projects: DCAT 3, Dublin Core, FOAF and the
//! few DSP, ODRL and DCAT-AP terms the profiles need.

/// Namespaces and the terms used by name across the crate.
pub struct Vocab;

impl Vocab {
    pub const DCAT: &'static str = "http://www.w3.org/ns/dcat#";
    pub const DCT: &'static str = "http://purl.org/dc/terms/";
    pub const FOAF: &'static str = "http://xmlns.com/foaf/0.1/";
    pub const DSPACE: &'static str = "https://w3id.org/dspace/2025/1/";
    pub const ODRL: &'static str = "http://www.w3.org/ns/odrl/2/";
    pub const XSD: &'static str = "http://www.w3.org/2001/XMLSchema#";
    pub const RDF: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    pub const RDFS: &'static str = "http://www.w3.org/2000/01/rdf-schema#";
    pub const ADMS: &'static str = "http://www.w3.org/ns/adms#";
    pub const VCARD: &'static str = "http://www.w3.org/2006/vcard/ns#";
    pub const SKOS: &'static str = "http://www.w3.org/2004/02/skos/core#";
    pub const LOCN: &'static str = "http://www.w3.org/ns/locn#";
    pub const SPDX: &'static str = "http://spdx.org/rdf/terms#";
    pub const OWL: &'static str = "http://www.w3.org/2002/07/owl#";
    pub const PROV: &'static str = "http://www.w3.org/ns/prov#";
    pub const DCATAP: &'static str = "http://data.europa.eu/r5r/";

    pub const DCAT_CATALOG: &'static str = "http://www.w3.org/ns/dcat#Catalog";
    pub const DCAT_DATASET: &'static str = "http://www.w3.org/ns/dcat#Dataset";
    pub const DCAT_DATASET_SERIES: &'static str = "http://www.w3.org/ns/dcat#DatasetSeries";
    pub const DCAT_DISTRIBUTION: &'static str = "http://www.w3.org/ns/dcat#Distribution";
    pub const DCAT_DATA_SERVICE: &'static str = "http://www.w3.org/ns/dcat#DataService";
    pub const DCAT_CATALOG_RECORD: &'static str = "http://www.w3.org/ns/dcat#CatalogRecord";
    pub const FOAF_AGENT: &'static str = "http://xmlns.com/foaf/0.1/Agent";
    pub const FOAF_ORGANIZATION: &'static str = "http://xmlns.com/foaf/0.1/Organization";
    pub const FOAF_PERSON: &'static str = "http://xmlns.com/foaf/0.1/Person";

    pub const DCAT_CATALOG_P: &'static str = "http://www.w3.org/ns/dcat#catalog";
    pub const DCAT_DATASET_P: &'static str = "http://www.w3.org/ns/dcat#dataset";
    pub const DCAT_SERVICE: &'static str = "http://www.w3.org/ns/dcat#service";
    pub const DCAT_RECORD: &'static str = "http://www.w3.org/ns/dcat#record";
    pub const DCAT_DISTRIBUTION_P: &'static str = "http://www.w3.org/ns/dcat#distribution";
    pub const DCAT_ACCESS_SERVICE: &'static str = "http://www.w3.org/ns/dcat#accessService";
    pub const DCAT_ACCESS_URL: &'static str = "http://www.w3.org/ns/dcat#accessURL";
    pub const DCAT_DOWNLOAD_URL: &'static str = "http://www.w3.org/ns/dcat#downloadURL";
    pub const DCAT_ENDPOINT_URL: &'static str = "http://www.w3.org/ns/dcat#endpointURL";
    pub const DCAT_ENDPOINT_DESCRIPTION: &'static str =
        "http://www.w3.org/ns/dcat#endpointDescription";
    pub const DCAT_SERVES_DATASET: &'static str = "http://www.w3.org/ns/dcat#servesDataset";
    pub const DCAT_KEYWORD: &'static str = "http://www.w3.org/ns/dcat#keyword";
    pub const DCAT_THEME: &'static str = "http://www.w3.org/ns/dcat#theme";
    pub const DCAT_THEME_TAXONOMY: &'static str = "http://www.w3.org/ns/dcat#themeTaxonomy";
    pub const DCAT_MEDIA_TYPE: &'static str = "http://www.w3.org/ns/dcat#mediaType";
    pub const DCAT_IN_SERIES: &'static str = "http://www.w3.org/ns/dcat#inSeries";
    pub const DCAT_CONTACT_POINT: &'static str = "http://www.w3.org/ns/dcat#contactPoint";
    pub const DCAT_LANDING_PAGE: &'static str = "http://www.w3.org/ns/dcat#landingPage";
    pub const DCAT_BYTE_SIZE: &'static str = "http://www.w3.org/ns/dcat#byteSize";
    pub const DCAT_VERSION: &'static str = "http://www.w3.org/ns/dcat#version";

    pub const DCT_TITLE: &'static str = "http://purl.org/dc/terms/title";
    pub const DCT_DESCRIPTION: &'static str = "http://purl.org/dc/terms/description";
    pub const DCT_ISSUED: &'static str = "http://purl.org/dc/terms/issued";
    pub const DCT_MODIFIED: &'static str = "http://purl.org/dc/terms/modified";
    pub const DCT_IDENTIFIER: &'static str = "http://purl.org/dc/terms/identifier";
    pub const DCT_FORMAT: &'static str = "http://purl.org/dc/terms/format";
    pub const DCT_CONFORMS_TO: &'static str = "http://purl.org/dc/terms/conformsTo";
    pub const DCT_CREATOR: &'static str = "http://purl.org/dc/terms/creator";
    pub const DCT_PUBLISHER: &'static str = "http://purl.org/dc/terms/publisher";
    pub const DCT_RIGHTS_HOLDER: &'static str = "http://purl.org/dc/terms/rightsHolder";
    pub const DCT_LANGUAGE: &'static str = "http://purl.org/dc/terms/language";
    pub const DCT_LICENSE: &'static str = "http://purl.org/dc/terms/license";
    pub const DCT_ACCESS_RIGHTS: &'static str = "http://purl.org/dc/terms/accessRights";
    pub const DCT_SPATIAL: &'static str = "http://purl.org/dc/terms/spatial";
    pub const DCT_TEMPORAL: &'static str = "http://purl.org/dc/terms/temporal";
    pub const DCT_ACCRUAL_PERIODICITY: &'static str = "http://purl.org/dc/terms/accrualPeriodicity";
    pub const DCT_TYPE: &'static str = "http://purl.org/dc/terms/type";
    pub const DCT_RIGHTS: &'static str = "http://purl.org/dc/terms/rights";
    pub const DCT_STANDARD: &'static str = "http://purl.org/dc/terms/Standard";
    pub const OWL_VERSION_INFO: &'static str = "http://www.w3.org/2002/07/owl#versionInfo";
    /// The data schema of a dataset or distribution; profiles publish it as `dct:conformsTo`.
    pub const EUNOMIA_SCHEMA: &'static str = "https://w3id.org/eunomia/v1/schema";
    /// `draft` keeps a dataset or distribution out of every profile; absent means published.
    pub const EUNOMIA_PUBLICATION_STATE: &'static str =
        "https://w3id.org/eunomia/v1/publicationState";
    pub const DRAFT: &'static str = "draft";
    pub const PUBLISHED: &'static str = "published";

    pub const FOAF_NAME: &'static str = "http://xmlns.com/foaf/0.1/name";
    pub const FOAF_HOMEPAGE: &'static str = "http://xmlns.com/foaf/0.1/homepage";
    pub const FOAF_PRIMARY_TOPIC: &'static str = "http://xmlns.com/foaf/0.1/primaryTopic";
    pub const ADMS_STATUS: &'static str = "http://www.w3.org/ns/adms#status";
    pub const DSPACE_PARTICIPANT_ID: &'static str = "https://w3id.org/dspace/2025/1/participantId";
    pub const ODRL_HAS_POLICY: &'static str = "http://www.w3.org/ns/odrl/2/hasPolicy";
    pub const XSD_DATE_TIME: &'static str = "http://www.w3.org/2001/XMLSchema#dateTime";
    pub const XSD_DATE: &'static str = "http://www.w3.org/2001/XMLSchema#date";
    pub const XSD_STRING: &'static str = "http://www.w3.org/2001/XMLSchema#string";
    pub const RDF_LANG_STRING: &'static str =
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
    pub const XSD_G_YEAR: &'static str = "http://www.w3.org/2001/XMLSchema#gYear";
    pub const XSD_G_YEAR_MONTH: &'static str = "http://www.w3.org/2001/XMLSchema#gYearMonth";

    /// Controlled vocabularies (EU NALs and IANA) the DCAT-AP profile binds properties to.
    pub const NAL_FILE_TYPE: &'static str =
        "http://publications.europa.eu/resource/authority/file-type/";
    pub const NAL_LANGUAGE: &'static str =
        "http://publications.europa.eu/resource/authority/language/";
    pub const NAL_DATA_THEME: &'static str =
        "http://publications.europa.eu/resource/authority/data-theme/";
    pub const NAL_FREQUENCY: &'static str =
        "http://publications.europa.eu/resource/authority/frequency/";
    pub const NAL_ACCESS_RIGHT: &'static str =
        "http://publications.europa.eu/resource/authority/access-right/";
    pub const NAL_DISTRIBUTION_STATUS: &'static str =
        "http://publications.europa.eu/resource/authority/distribution-status/";
    pub const IANA_MEDIA_TYPES: &'static str = "https://www.iana.org/assignments/media-types/";

    /// Links whose object is another catalog resource; a `urn:` object must exist locally.
    pub const RESOURCE_LINKS: [&'static str; 7] = [
        Self::DCAT_ACCESS_SERVICE,
        Self::DCAT_SERVES_DATASET,
        Self::DCT_PUBLISHER,
        Self::DCT_CREATOR,
        Self::DCT_RIGHTS_HOLDER,
        Self::DCAT_IN_SERIES,
        Self::FOAF_PRIMARY_TOPIC,
    ];

    /// Properties the agent sets itself; whatever a caller sends for them is replaced.
    pub const MANAGED: [&'static str; 2] = [Self::DCT_ISSUED, Self::DCT_MODIFIED];

    /// The common prefixes, for compacting an IRI no term covers.
    pub const PREFIXES: [(&'static str, &'static str); 17] = [
        ("dcat", Self::DCAT),
        ("dct", Self::DCT),
        ("foaf", Self::FOAF),
        ("dspace", Self::DSPACE),
        ("odrl", Self::ODRL),
        ("xsd", Self::XSD),
        ("rdf", Self::RDF),
        ("rdfs", Self::RDFS),
        ("adms", Self::ADMS),
        ("vcard", Self::VCARD),
        ("skos", Self::SKOS),
        ("locn", Self::LOCN),
        ("spdx", Self::SPDX),
        ("owl", Self::OWL),
        ("prov", Self::PROV),
        ("dcatap", Self::DCATAP),
        ("eunomia", "https://w3id.org/eunomia/v1/"),
    ];

    /// `dct:title` rather than the whole IRI, for messages; an IRI no prefix covers stays whole.
    pub fn compact(iri: &str) -> String {
        Self::PREFIXES
            .iter()
            .filter(|(_, ns)| iri.starts_with(ns))
            .max_by_key(|(_, ns)| ns.len())
            .map(|(prefix, ns)| format!("{prefix}:{}", &iri[ns.len()..]))
            .unwrap_or_else(|| iri.to_string())
    }
}
