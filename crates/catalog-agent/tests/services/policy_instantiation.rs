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

//! PolicyInstantiationService over mocked template and offer services: parameters filled into
//! the template and the offer stored with its provenance.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use catalog_agent::entities::odrl_policies::CatalogEntityTypes;
use catalog_agent::entities::policy_instantiation::NewPolicyInstantiationDto;
use catalog_agent::entities::policy_templates::PolicyTemplateAllowedDefaultValues;
use catalog_agent::entities::policy_templates::PolicyTemplateAllowedDefaultValues::Stringable;
use catalog_agent::services::odrl_policies::MockOdrlPolicyServiceTrait;
use catalog_agent::services::policy_instantiation::service::PolicyInstantiationService;
use catalog_agent::services::policy_instantiation::PolicyInstantiationServiceTrait;
use catalog_agent::services::policy_templates::MockPolicyTemplateServiceTrait;
use common::test_utils::scopes::TestScopes;
use serde_json::json;
use urn::Urn;
use ymir::errors::Errors;

use crate::support::builders::{odrl_policy_dto, policy_template};

const ENTITY: &str = "urn:uuid:00000000-0000-0000-0000-000000000100";

fn request(params: &[(&str, PolicyTemplateAllowedDefaultValues)]) -> NewPolicyInstantiationDto {
    NewPolicyInstantiationDto {
        id: "tpl-1".to_string(),
        version: "1".to_string(),
        parameters: params
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<HashMap<_, _>>(),
        entity_id: Urn::from_str(ENTITY).unwrap(),
        entity_type: CatalogEntityTypes::Dataset,
        description: Some("EU only".to_string()),
    }
}

/// Templates service that returns a template whose constraints use `$region` as a string
/// operand and `$maxUses` inside a typed literal, `$maxUses` defaulting to 3.
fn templates() -> MockPolicyTemplateServiceTrait {
    let mut templates = MockPolicyTemplateServiceTrait::new();
    templates
        .expect_get_policies_template_by_version_and_id()
        .withf(|_, id, version| id == "tpl-1" && version == "1")
        .returning(|_, _, _| {
            Ok(policy_template(
                json!({
                    "$region": {"dataType": "selection", "restrictions": {"values": ["eu", "us"]}},
                    "$maxUses": {"dataType": "integer", "defaultValue": 3}
                }),
                json!({"permission": [{"action": "use", "constraint": [
                    {"leftOperand": "spatial", "operator": "eq", "rightOperand": "$region"},
                    {"leftOperand": "count", "operator": "lteq",
                     "rightOperand": {"@value": "$maxUses", "@type": "xsd:integer"}}
                ]}]}),
            ))
        });
    templates
}

/// Request values and defaults replace the placeholders, and the offer records which template,
/// version and parameters produced it.
#[tokio::test]
async fn parameters_fill_the_template_and_provenance_is_kept() {
    let mut offers = MockOdrlPolicyServiceTrait::new();
    offers
        .expect_create_odrl_offer()
        .withf(|_, p| {
            let offer = serde_json::to_value(&p.odrl_offer).unwrap();
            let constraints = &offer["permission"][0]["constraint"];
            constraints[0]["rightOperand"] == "eu"
                && constraints[1]["rightOperand"]["@value"] == 3.0
                && p.tenant_id.as_deref() == Some("tenant-1")
                && p.entity_id.to_string() == ENTITY
                && p.source_template_id.as_deref() == Some("tpl-1")
                && p.source_template_version.as_deref() == Some("1")
                && p.instantiation_parameters == Some(json!({"$region": "eu"}))
                && p.description.as_deref() == Some("EU only")
        })
        .times(1)
        .returning(|_, _| Ok(odrl_policy_dto(400, "Dataset")));
    let svc = PolicyInstantiationService::new(Arc::new(offers), Arc::new(templates()));

    let created = svc
        .instantiate_policy(
            &TestScopes::owner("tenant-1"),
            &request(&[("$region", Stringable("eu".into()))]),
        )
        .await
        .unwrap();
    assert_eq!(created.inner.entity_type, "Dataset");
}

/// An invalid request never reaches the offer service.
#[tokio::test]
async fn invalid_request_creates_nothing() {
    let svc = PolicyInstantiationService::new(
        Arc::new(MockOdrlPolicyServiceTrait::new()),
        Arc::new(templates()),
    );
    let result = svc
        .instantiate_policy(
            &TestScopes::owner("tenant-1"),
            &request(&[("$region", Stringable("asia".into()))]),
        )
        .await;
    assert!(result.is_err());
}

/// A template that cannot be read fails the instantiation.
#[tokio::test]
async fn missing_template_fails_the_instantiation() {
    let mut templates = MockPolicyTemplateServiceTrait::new();
    templates
        .expect_get_policies_template_by_version_and_id()
        .returning(|_, _, _| Err(Errors::missing_resource("tpl-1", "not found", None)));
    let svc = PolicyInstantiationService::new(
        Arc::new(MockOdrlPolicyServiceTrait::new()),
        Arc::new(templates),
    );

    let result = svc
        .instantiate_policy(&TestScopes::owner("tenant-1"), &request(&[]))
        .await;
    assert!(result.is_err());
}

/// A numeric parameter used as a bare right operand fills the template too.
#[tokio::test]
async fn numeric_parameter_as_bare_right_operand() {
    let mut templates = MockPolicyTemplateServiceTrait::new();
    templates
        .expect_get_policies_template_by_version_and_id()
        .returning(|_, _, _| {
            Ok(policy_template(
                json!({"$maxUses": {"dataType": "integer", "defaultValue": 3}}),
                json!({"permission": [{"action": "use", "constraint": [
                    {"leftOperand": "count", "operator": "lteq", "rightOperand": "$maxUses"}
                ]}]}),
            ))
        });
    let mut offers = MockOdrlPolicyServiceTrait::new();
    offers
        .expect_create_odrl_offer()
        .returning(|_, _| Ok(odrl_policy_dto(400, "Dataset")));
    let svc = PolicyInstantiationService::new(Arc::new(offers), Arc::new(templates));

    let result = svc
        .instantiate_policy(&TestScopes::owner("tenant-1"), &request(&[]))
        .await;
    assert!(result.is_ok(), "{result:?}");
}
