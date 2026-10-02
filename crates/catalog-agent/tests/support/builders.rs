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

//! DTOs of each catalog entity, numbered by URN and owned by the stub tenant.

use catalog_agent::data::entities::{catalog, dataservice, dataset, distribution, odrl_offer};
use catalog_agent::entities::catalogs::CatalogDto;
use catalog_agent::entities::data_services::DataServiceDto;
use catalog_agent::entities::datasets::DatasetDto;
use catalog_agent::entities::distributions::DistributionDto;
use catalog_agent::entities::odrl_policies::OdrlPolicyDto;
use chrono::Utc;
use common::test_utils::grpc::TENANT;
use serde_json::json;

use super::fixtures::urn;

/// Catalog `n`; catalog 1 is the main one.
pub fn catalog_dto(n: u32) -> CatalogDto {
    CatalogDto {
        inner: catalog::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            foaf_home_page: None,
            dct_conforms_to: None,
            dct_creator: Some("creator".into()),
            dct_identifier: None,
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: Some(format!("catalog-{n}")),
            dspace_participant_id: None,
            dspace_main_catalog: n == 1,
        },
    }
}

/// Main data service `n` of catalog 100.
pub fn data_service_dto(n: u32) -> DataServiceDto {
    DataServiceDto {
        inner: dataservice::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            dcat_endpoint_description: None,
            dcat_endpoint_url: "https://svc.example".into(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_identifier: None,
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: None,
            dct_description: None,
            catalog_id: urn(100),
            dspace_main_data_service: true,
        },
    }
}

/// Dataset `n` of catalog 100.
pub fn dataset_dto(n: u32) -> DatasetDto {
    DatasetDto {
        inner: dataset::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_identifier: None,
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: Some(format!("dataset-{n}")),
            dct_description: None,
            catalog_id: urn(100),
        },
    }
}

/// Distribution `n` of dataset 100, served by data service 200.
pub fn distribution_dto(n: u32) -> DistributionDto {
    DistributionDto {
        inner: distribution::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: None,
            dct_description: None,
            dcat_access_service: urn(200),
            dataset_id: urn(100),
            dct_format: Some("HTTP_PULL".into()),
        },
    }
}

/// ODRL offer `n` on entity 100 of `entity_type`.
pub fn odrl_policy_dto(n: u32, entity_type: &str) -> OdrlPolicyDto {
    OdrlPolicyDto {
        inner: odrl_offer::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            odrl_offer: json!({"permission": [{"action": "use"}]}),
            entity: urn(100),
            entity_type: entity_type.to_string(),
            created_at: Utc::now().into(),
            source_template_id: None,
            source_template_version: None,
            instantiation_parameters: Some(json!({"n": 3})),
            description: None,
        },
    }
}

/// Template `tpl-1` v1 of the stub tenant, built from its JSON `parameters` and ODRL `content`.
pub fn policy_template(
    parameters: serde_json::Value,
    content: serde_json::Value,
) -> catalog_agent::entities::policy_templates::PolicyTemplateDto {
    serde_json::from_value(json!({
        "id": "tpl-1",
        "tenantId": TENANT,
        "version": "1",
        "date": "2026-01-01T00:00:00Z",
        "author": "tests",
        "content": content,
        "parameters": parameters,
    }))
    .expect("valid policy template")
}

/// Minimal DSP catalog `id`, as a peer would publish it.
pub fn peer_catalog(id: &str) -> catalog_agent::protocols::dsp::types::catalog_definition::Catalog {
    serde_json::from_value(json!({
        "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
        "@type": "Catalog",
        "@id": id,
        "identifier": id,
        "issued": "2026-01-01T00:00:00",
        "description": [],
        "catalog": [],
        "dataset": [],
        "service": [],
    }))
    .expect("valid catalog")
}

/// Participant `participant_id` known by the stub tenant.
pub fn mate(participant_id: &str) -> ymir::data::entities::shared::participant::Model {
    ymir::data::entities::shared::participant::Model {
        tenant_id: TENANT.to_string(),
        participant_id: participant_id.to_string(),
        participant_nick: participant_id.to_string(),
        participant_type: ymir::types::participants::ParticipantType::Agent,
        base_url: format!("https://{participant_id}.example"),
        token: None,
        saved_at: Utc::now(),
        last_interaction: Utc::now(),
        extra_fields: json!({}),
    }
}
