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

//! The data schema in every profile: published as `dct:conformsTo`, inherited by distributions
//! without their own, never leaking the internal predicate.

use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::resource::CatalogResource;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::resource_set::ResourceSet;
use catalog_agent::entities::vocab::Vocab;
use catalog_agent::profiles::dcat3::Dcat3;
use catalog_agent::profiles::dcat_ap::DcatAp;
use catalog_agent::profiles::dsp::DspProfileAdapter;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::profiles::shared::effective::EffectiveGraph;
use catalog_agent::profiles::ProfileAdapterTrait;
use serde_json::Value;

use crate::support::builders::{
    data_service, dataset, distribution, id, main_catalog, offer, resource, titled, urn,
};

const DATASET_SCHEMA: &str = "https://provider.example/public/schemas/urn:schema:a/1";
const OWN_SCHEMA: &str = "https://example.org/schemas/readings.json";

fn with_schema(mut resource: CatalogResource, schema: &str) -> CatalogResource {
    let mut description = resource.description().clone();
    description.put(Vocab::EUNOMIA_SCHEMA, vec![ResourceGraph::iri(schema)]);
    resource = resource.revise(description);
    resource
}

/// Dataset 3 with a schema; distribution 4 inherits it, distribution 5 has its own.
fn schema_set() -> (CatalogResource, ResourceSet) {
    let main = main_catalog(1);
    let mut set = ResourceSet::new();
    set.insert(main.clone());
    set.insert(data_service(2, 1));
    set.insert(with_schema(dataset(3, 1), DATASET_SCHEMA));
    set.insert(distribution(4, 3, 2, "http+pull"));
    set.insert(with_schema(distribution(5, 3, 2, "http+push"), OWN_SCHEMA));
    set.insert_offers([offer(1, &urn(ResourceKind::Dataset, 3))]);
    (main, set)
}

fn conforms_to(node: &Value) -> Vec<String> {
    let values = match &node["dct:conformsTo"] {
        Value::Array(values) => values.clone(),
        Value::Null => vec![],
        one => vec![one.clone()],
    };
    values
        .iter()
        .filter_map(|v| {
            v.get("@id")
                .or(Some(v))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

/// The dataset publishes its schema; distributions inherit it or keep their own.
#[test]
fn dsp_publishes_own_and_inherited_schemas() {
    let (_, set) = schema_set();
    let dataset = set.get(&id(ResourceKind::Dataset, 3)).unwrap();
    let document = DspProfileAdapter::new().project(&set, dataset).unwrap();

    assert_eq!(conforms_to(&document), vec![DATASET_SCHEMA]);
    let distributions = document["distribution"].as_array().unwrap();
    let by_format = |format: &str| {
        distributions
            .iter()
            .find(|d| d["format"] == format)
            .unwrap()
            .clone()
    };
    assert_eq!(conforms_to(&by_format("http+pull")), vec![DATASET_SCHEMA]);
    assert_eq!(conforms_to(&by_format("http+push")), vec![OWN_SCHEMA]);
    assert!(!document.to_string().contains(Vocab::EUNOMIA_SCHEMA));
}

/// A dataset with a schema still meets the DSP schemas.
#[test]
fn dsp_dataset_with_schema_conforms() {
    let (_, set) = schema_set();
    let dataset = set.get(&id(ResourceKind::Dataset, 3)).unwrap();
    let diagnosis = DspProfileAdapter::new()
        .diagnose(&set, dataset, true)
        .unwrap();

    assert!(diagnosis.compliant, "{:#?}", diagnosis.findings);
}

/// A distribution loaded alone sees its dataset as a parent, which is not projected.
#[test]
fn distribution_alone_inherits_from_held_parent() {
    let mut set = ResourceSet::new();
    let alone = distribution(4, 3, 2, "http+pull");
    set.insert(alone.clone());
    set.insert_parent(with_schema(dataset(3, 1), DATASET_SCHEMA));
    let graph = EffectiveGraph::of(&alone, &set);

    assert_eq!(graph.iris(Vocab::DCT_CONFORMS_TO), vec![DATASET_SCHEMA]);
    let projected = DcatAdapter::<Dcat3>::default()
        .project(&set, &alone)
        .unwrap();
    assert!(!projected
        .to_string()
        .contains(&id(ResourceKind::Dataset, 3)));
}

/// Other standards stay; the schema comes first, typed as a standard, and is not repeated.
#[test]
fn schema_joins_other_standards_once() {
    let mut description = titled("Readings");
    description.put(
        Vocab::DCT_CONFORMS_TO,
        vec![
            ResourceGraph::iri("https://example.org/model"),
            ResourceGraph::iri(DATASET_SCHEMA),
        ],
    );
    description.put(
        Vocab::EUNOMIA_SCHEMA,
        vec![ResourceGraph::iri(DATASET_SCHEMA)],
    );
    let dataset = resource(ResourceKind::Dataset, 3, Some(1), None, description);
    let graph = EffectiveGraph::of(&dataset, &ResourceSet::new());

    assert_eq!(
        graph.iris(Vocab::DCT_CONFORMS_TO),
        vec![DATASET_SCHEMA, "https://example.org/model"]
    );
    let first = &graph.values(Vocab::DCT_CONFORMS_TO)[0];
    assert_eq!(first["@type"][0], Vocab::DCT_STANDARD);
    assert!(!graph.has(Vocab::EUNOMIA_SCHEMA));
}

/// A distribution with no schema of its own and a dataset without one publishes none.
#[test]
fn no_schema_no_standard() {
    let mut plain = ResourceSet::new();
    plain.insert(dataset(6, 1));
    let alone = distribution(7, 6, 2, "http+pull");
    plain.insert(alone.clone());
    assert!(!EffectiveGraph::of(&alone, &plain).has(Vocab::DCT_CONFORMS_TO));
}

/// The DCAT-AP shapes accept the schema as a linked schema of the distribution.
#[test]
fn dcat_ap_accepts_the_schema_standard() {
    let (_, set) = schema_set();
    let adapter = DcatAdapter::<DcatAp>::default();
    for n in [4, 5] {
        let distribution = set.get(&id(ResourceKind::Distribution, n)).unwrap();
        let diagnosis = adapter.diagnose(&set, distribution, false).unwrap();
        let about_schema: Vec<_> = diagnosis
            .findings
            .iter()
            .filter(|f| f.property.as_deref() == Some(Vocab::DCT_CONFORMS_TO))
            .collect();
        assert!(about_schema.is_empty(), "{about_schema:#?}");
    }
}
