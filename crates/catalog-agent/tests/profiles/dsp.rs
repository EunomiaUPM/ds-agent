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

//! DspProfileAdapter over built resource sets; the DSP schemas are the published ones.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::resource_set::ResourceSet;
use catalog_agent::entities::vocab::Vocab;
use catalog_agent::profiles::dsp::rules::DspRules;
use catalog_agent::profiles::dsp::DspProfileAdapter;
use catalog_agent::profiles::{ProfileAdapterTrait, Severity};
use common::dsp_common::rdf::DspProfile;
use serde_json::{json, Value};

use crate::support::builders::{data_service, dataset, distribution, id, main_catalog, offer, provider_set, urn};

/// A catalog where every dataset has an offer and a distribution.
fn compliant_set() -> (catalog_agent::entities::resource::CatalogResource, ResourceSet) {
    let main = main_catalog(1);
    let mut set = ResourceSet::new();
    set.insert(main.clone());
    set.insert(data_service(2, 1));
    set.insert(dataset(3, 1));
    set.insert(distribution(4, 3, 2, "http+pull"));
    set.insert_offers([offer(1, &urn(ResourceKind::Dataset, 3))]);
    (main, set)
}

/// The main catalog comes in the DSP shape: identity, datasets with offers and distributions,
/// services, and the other catalogs nested.
#[test]
fn catalog_has_the_dsp_shape() {
    let (main, set) = provider_set();
    let catalog = DspProfileAdapter::new().project(&set, &main).unwrap();

    assert_eq!(catalog["@context"], json!(["https://w3id.org/dspace/2025/1/context.jsonld"]));
    assert_eq!(catalog["@type"], "Catalog");
    assert_eq!(catalog["participantId"], "did:web:provider.example");
    assert_eq!(catalog["service"][0]["endpointURL"], "https://provider.example/dsp");
    let dataset = &catalog["dataset"][0];
    assert_eq!(dataset["@id"], id(ResourceKind::Dataset, 3));
    assert_eq!(dataset["hasPolicy"][0]["@type"], "Offer");
    assert_eq!(dataset["distribution"][0]["format"], "http+pull");
    assert_eq!(dataset["distribution"][0]["accessService"]["@id"], id(ResourceKind::DataService, 2));
    assert_eq!(catalog["catalog"][0]["dataset"][0]["@id"], id(ResourceKind::Dataset, 6));
}

/// Properties DSP has no term for go out as compact IRIs and stay RDF.
#[tokio::test]
async fn other_properties_expand_as_dcat_triples() {
    let (main, set) = provider_set();
    let catalog = DspProfileAdapter::new().project(&set, &main).unwrap();
    assert!(catalog["dataset"][0]["dct:title"].is_object());

    let quads = DspProfile::shared().expand(&catalog).await.unwrap().canonical_n_quads;
    assert!(quads.contains(&format!("<{}> <{}> \"dataset-3\"@en", id(ResourceKind::Dataset, 3), Vocab::DCT_TITLE)));
    assert!(quads.contains(&format!("<{}> <{}> <{}>", id(ResourceKind::Catalog, 1), Vocab::DCAT_DATASET_P, id(ResourceKind::Dataset, 3))));
}

/// A catalog of datasets with offers and distributions conforms, schema included.
#[test]
fn complete_catalog_conforms() {
    let (main, set) = compliant_set();
    let diagnosis = DspProfileAdapter::new().diagnose(&set, &main, true).unwrap();

    assert!(diagnosis.compliant, "{:#?}", diagnosis.findings);
}

/// A dataset with no offer breaks DSP: by the rules and by the schema.
#[test]
fn dataset_without_offer_is_a_violation() {
    let (_, mut set) = compliant_set();
    let lonely = dataset(9, 1);
    set.insert(lonely.clone());
    let diagnosis = DspProfileAdapter::new().diagnose(&set, &lonely, false).unwrap();

    assert!(!diagnosis.compliant);
    assert!(diagnosis.findings.iter().any(|f| f.severity == Severity::Violation
        && f.property.as_deref() == Some(Vocab::ODRL_HAS_POLICY)));
    assert!(diagnosis.findings.iter().any(|f| f.message.starts_with("DSP schema")));
}

/// A distribution with no access service or format breaks DSP.
#[test]
fn distribution_needs_format_and_service() {
    let (_, mut set) = compliant_set();
    let bare = catalog_agent::entities::resource::CatalogResource::create(
        urn(ResourceKind::Distribution, 8),
        ResourceKind::Distribution,
        ymir::oauth::Owner::connector(),
        None,
        Some(urn(ResourceKind::Dataset, 3)),
        false,
        Default::default(),
    );
    set.insert(bare.clone());
    let diagnosis = DspProfileAdapter::new().diagnose(&set, &bare, false).unwrap();

    let properties: Vec<_> = diagnosis.findings.iter().filter_map(|f| f.property.as_deref()).collect();
    assert!(properties.contains(&Vocab::DCT_FORMAT));
    assert!(properties.contains(&Vocab::DCAT_ACCESS_SERVICE));
}

/// The published DSP examples pass the schemas the doctor uses.
#[test]
fn published_examples_pass_the_schemas() {
    let rules = DspRules::new();
    let catalog: Value = serde_json::from_str(include_str!("../../dsp_schema/example/catalog.json")).unwrap();
    let dataset: Value = serde_json::from_str(include_str!("../../dsp_schema/example/dataset.json")).unwrap();

    assert!(rules.check_schema(ResourceKind::Catalog, "c", &catalog).is_empty());
    assert!(rules.check_schema(ResourceKind::Dataset, "d", &dataset).is_empty());
}
