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

//! PolicyCompositionService over mocked repositories: composed offers keep plain ODRL, their
//! parts and the templates used; instantiating is a composition of one part.

use catalog_agent::data::repo::odrl_offer::MockOdrlOfferRepoTrait;
use catalog_agent::data::repo::policy_template::MockPolicyTemplateRepoTrait;
use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::policy_composition::NewPolicyCompositionDto;
use catalog_agent::entities::policy_instantiation::NewPolicyInstantiationDto;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::services::odrl_policies::views::OdrlPolicyView;
use catalog_agent::entities::policy_templates::ControlPoint;
use common::test_utils::scopes::TestUsers;
use serde_json::json;

use crate::support::builders::{dataset, offer, piece, urn};
use crate::support::context::TestContext;

fn templates() -> MockPolicyTemplateRepoTrait {
    let mut repo = MockPolicyTemplateRepoTrait::new();
    repo.expect_get_by_keys().returning(|_, keys| {
        Ok(keys
            .iter()
            .filter_map(|(id, _)| match id.as_str() {
                "use" => Some(piece("use", "rule", json!({"permission": {"action": "use"}}), json!({}), json!(["transfer"]))),
                "expires-after" => Some(piece(
                    "expires-after",
                    "constraint",
                    json!({"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "$duration"}),
                    json!({"$duration": {"dataType": "duration", "defaultValue": "P30D"}}),
                    json!(["validity"]),
                )),
                _ => None,
            })
            .collect())
    });
    repo
}

fn resources() -> MockResourceRepoTrait {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(dataset(3, 1))));
    repo
}

/// A composed offer stores plain ODRL, its parts, its control points and the templates used.
#[tokio::test]
async fn composed_offer_keeps_plain_odrl_and_its_parts() {
    let mut offers = MockOdrlOfferRepoTrait::new();
    offers.expect_get_batch().returning(|_, ids| Ok(ids.iter().map(|id| {
        let mut o = offer(9, &urn(ResourceKind::Dataset, 3));
        o.id = id.clone();
        o.odrl_offer = json!({"obligation": [{"action": "delete"}]});
        o
    }).collect()));
    offers
        .expect_insert()
        .withf(|_, refs| refs == [("expires-after".to_string(), "1".to_string()), ("use".to_string(), "1".to_string())])
        .returning(|o, _| Ok(o.clone()));
    let ctx = TestContext::new()
        .template_repo(templates())
        .resource_repo(resources())
        .offer_repo(offers)
        .build();
    let request: NewPolicyCompositionDto = serde_json::from_value(json!({
        "entityId": urn(ResourceKind::Dataset, 3).to_string(),
        "entityType": "Dataset",
        "parts": [
            {"@ref": "use@1"},
            {"@ref": "expires-after@1", "into": "permission"},
            {"@offer": "urn:odrl-policy:00000000-0000-0000-0000-000000000009"}
        ]
    }))
    .unwrap();

    let stored = ctx.composition().compose(&TestUsers::alone("ana"), &request).await.unwrap();
    assert_eq!(
        stored.odrl_offer,
        json!({
            "permission": [{"action": "use", "constraint": [
                {"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "P30D"}
            ]}],
            "obligation": [{"action": "delete"}]
        })
    );
    let view = OdrlPolicyView::assemble(stored);
    assert_eq!(view.composition, Some(json!(request.parts)));
    assert_eq!(view.control_points, vec![ControlPoint::Transfer, ControlPoint::Validity]);
}

/// Parts that do not add up to valid ODRL are refused unstored.
#[tokio::test]
async fn invalid_composition_is_refused() {
    let ctx = TestContext::new().template_repo(templates()).build();
    let preview = ctx
        .composition()
        .preview(&TestUsers::alone("ana"), &[json!({"permission": [{"noAction": true}]})])
        .await
        .unwrap_err();
    assert_eq!(preview.info().status_code.as_u16(), 400);
}

/// Instantiating a template keeps it as the offer's source, with the values given.
#[tokio::test]
async fn instantiation_is_a_one_part_composition() {
    let mut repo = MockPolicyTemplateRepoTrait::new();
    repo.expect_get_by_keys().returning(|_, _| {
        Ok(vec![piece(
            "research",
            "policy",
            json!({"permission": [{"action": "use", "constraint": [
                {"leftOperand": "odrl:purpose", "operator": "eq", "rightOperand": "$purpose"}
            ]}]}),
            json!({"$purpose": {"dataType": "string"}}),
            json!([]),
        )])
    });
    let mut offers = MockOdrlOfferRepoTrait::new();
    offers.expect_insert().returning(|o, _| Ok(o.clone()));
    let ctx = TestContext::new().template_repo(repo).resource_repo(resources()).offer_repo(offers).build();
    let request: NewPolicyInstantiationDto = serde_json::from_value(json!({
        "id": "research",
        "version": "1",
        "parameters": {"$purpose": "science"},
        "entityId": urn(ResourceKind::Dataset, 3).to_string(),
        "entityType": "Dataset",
        "description": null
    }))
    .unwrap();

    let stored = ctx.composition().instantiate(&TestUsers::alone("ana"), &request).await.unwrap();
    assert_eq!(stored.source_template_id.as_deref(), Some("research"));
    assert_eq!(stored.instantiation_parameters, Some(json!({"$purpose": "science"})));
    assert_eq!(stored.odrl_offer["permission"][0]["constraint"][0]["rightOperand"], json!("science"));
}
