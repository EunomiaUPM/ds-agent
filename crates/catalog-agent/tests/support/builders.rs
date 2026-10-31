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

//! Resources, offers and graphs numbered by URN, as tests need them.

use std::str::FromStr;

use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::odrl_offer::OdrlOffer;
use catalog_agent::entities::resource::CatalogResource;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::resource_set::ResourceSet;
use catalog_agent::entities::vocab::Vocab;
use serde_json::json;
use urn::Urn;
use ymir::oauth::Owner;

/// The URN of resource `n` of `kind`, e.g. `urn:dataset:000…007`.
pub fn urn(kind: ResourceKind, n: u32) -> Urn {
    Urn::from_str(&format!(
        "urn:{}:00000000-0000-0000-0000-{n:012}",
        kind.urn_nid()
    ))
    .unwrap()
}

pub fn id(kind: ResourceKind, n: u32) -> String {
    urn(kind, n).to_string()
}

/// A description with an English title.
pub fn titled(title: &str) -> ResourceGraph {
    let mut graph = ResourceGraph::default();
    graph.put(
        Vocab::DCT_TITLE,
        vec![ResourceGraph::literal(title, Some("en"))],
    );
    graph
}

/// A stored resource of `kind`, the connector's (public).
pub fn resource(
    kind: ResourceKind,
    n: u32,
    catalog: Option<u32>,
    dataset: Option<u32>,
    description: ResourceGraph,
) -> CatalogResource {
    CatalogResource::create(
        urn(kind, n),
        kind,
        Owner::connector(),
        catalog.map(|c| urn(ResourceKind::Catalog, c)),
        dataset.map(|d| urn(ResourceKind::Dataset, d)),
        false,
        description,
    )
}

/// The main catalog `n`, with the provider's participant id.
pub fn main_catalog(n: u32) -> CatalogResource {
    let mut description = titled("Main catalog");
    description.put(
        Vocab::DSPACE_PARTICIPANT_ID,
        vec![ResourceGraph::iri("did:web:provider.example")],
    );
    CatalogResource::create(
        urn(ResourceKind::Catalog, n),
        ResourceKind::Catalog,
        Owner::connector(),
        None,
        None,
        true,
        description,
    )
}

pub fn catalog(n: u32) -> CatalogResource {
    resource(
        ResourceKind::Catalog,
        n,
        None,
        None,
        titled(&format!("catalog-{n}")),
    )
}

pub fn dataset(n: u32, catalog: u32) -> CatalogResource {
    resource(
        ResourceKind::Dataset,
        n,
        Some(catalog),
        None,
        titled(&format!("dataset-{n}")),
    )
}

pub fn data_service(n: u32, catalog: u32) -> CatalogResource {
    let mut description = titled(&format!("service-{n}"));
    description.put(
        Vocab::DCAT_ENDPOINT_URL,
        vec![ResourceGraph::iri("https://provider.example/dsp")],
    );
    resource(
        ResourceKind::DataService,
        n,
        Some(catalog),
        None,
        description,
    )
}

/// Distribution `n` of `dataset` in `format`, served by data service `service`.
pub fn distribution(n: u32, dataset: u32, service: u32, format: &str) -> CatalogResource {
    let mut description = titled(&format!("distribution-{n}"));
    description.put(
        Vocab::DCT_FORMAT,
        vec![ResourceGraph::literal(format, None)],
    );
    description.put(
        Vocab::DCAT_ACCESS_SERVICE,
        vec![ResourceGraph::iri(&id(ResourceKind::DataService, service))],
    );
    resource(
        ResourceKind::Distribution,
        n,
        None,
        Some(dataset),
        description,
    )
}

pub fn agent(n: u32, name: &str) -> CatalogResource {
    let mut description = ResourceGraph::default();
    description.put(
        Vocab::FOAF_NAME,
        vec![ResourceGraph::literal(name, Some("en"))],
    );
    resource(ResourceKind::Agent, n, None, None, description)
}

/// An offer `n` on `entity` permitting `use`.
pub fn offer(n: u32, entity: &Urn) -> OdrlOffer {
    OdrlOffer {
        id: Urn::from_str(&format!("urn:odrl-policy:00000000-0000-0000-0000-{n:012}")).unwrap(),
        owner: Owner::connector(),
        odrl_offer: json!({ "permission": [{ "action": "use" }] }),
        entity: entity.clone(),
        entity_type: "Dataset".to_string(),
        created_at: chrono::Utc::now(),
        source_template_id: None,
        source_template_version: None,
        instantiation_parameters: None,
        description: None,
        composition: None,
    }
}

/// The main catalog 1 with service 2, dataset 3 (an offer, distribution 4 in `http+pull`) and
/// sub-catalog 5 with dataset 6.
pub fn provider_set() -> (CatalogResource, ResourceSet) {
    let main = main_catalog(1);
    let mut set = ResourceSet::new();
    set.insert(main.clone());
    set.insert(data_service(2, 1));
    set.insert(dataset(3, 1));
    set.insert(distribution(4, 3, 2, "http+pull"));
    set.insert(catalog(5));
    set.insert(dataset(6, 5));
    set.insert_offers([offer(1, &urn(ResourceKind::Dataset, 3))]);
    (main, set)
}

/// The provider set with dataset 3 kept as a draft.
pub fn provider_set_with_draft() -> (CatalogResource, ResourceSet) {
    let (main, full) = provider_set();
    let mut set = ResourceSet::new();
    for resource in full.iter() {
        let draft = resource.id() == &urn(ResourceKind::Dataset, 3);
        set.insert(if draft {
            resource.with_publication(true)
        } else {
            resource.clone()
        });
    }
    set.insert_offers([offer(1, &urn(ResourceKind::Dataset, 3))]);
    (main, set)
}

/// Template `tpl-1` v1 with the given parameters and ODRL content.
pub fn policy_template(
    parameters: serde_json::Value,
    content: serde_json::Value,
) -> catalog_agent::entities::policy_templates::PolicyTemplateDto {
    serde_json::from_value(json!({
        "id": "tpl-1",
        "userId": "user",
        "userRole": "/admin",
        "visibility": "Public",
        "version": "1",
        "date": "2026-01-01T00:00:00Z",
        "author": "tests",
        "content": content,
        "parameters": parameters,
    }))
    .expect("valid policy template")
}

/// Template version `id@1` of the given kind, content, parameters and control points.
pub fn piece(
    id: &str,
    kind: &str,
    content: serde_json::Value,
    parameters: serde_json::Value,
    control_points: serde_json::Value,
) -> catalog_agent::entities::policy_templates::PolicyTemplateDto {
    serde_json::from_value(json!({
        "id": id,
        "userId": "connector",
        "userRole": "/admin",
        "visibility": "Public",
        "version": "1",
        "kind": kind,
        "controlPoints": control_points,
        "date": "2026-01-01T00:00:00Z",
        "author": "tests",
        "content": content,
        "parameters": parameters,
    }))
    .expect("valid template piece")
}
