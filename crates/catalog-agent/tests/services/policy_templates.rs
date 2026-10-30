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

//! PolicyTemplateService over a mocked template repository: references stored with a version,
//! refused deletes of referenced versions, usages and previews.

use catalog_agent::data::repo::odrl_offer::MockOdrlOfferRepoTrait;
use catalog_agent::data::repo::policy_template::MockPolicyTemplateRepoTrait;
use catalog_agent::entities::policy_composition::TemplatePreviewDto;
use catalog_agent::entities::policy_templates::{
    ControlPoint, NewPolicyTemplateDto, PolicyTemplateDto,
};
use common::test_utils::scopes::TestUsers;
use serde_json::json;

use crate::support::builders::piece;
use crate::support::context::TestContext;

fn expiry() -> PolicyTemplateDto {
    piece(
        "expires-after",
        "constraint",
        json!({"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "$duration"}),
        json!({"$duration": {"dataType": "duration"}}),
        json!(["validity"]),
    )
}

/// A repo that knows `expires-after@1` and stores whatever it is given.
fn repo_with_expiry() -> MockPolicyTemplateRepoTrait {
    let mut repo = MockPolicyTemplateRepoTrait::new();
    repo.expect_get_by_keys().returning(|_, keys| {
        Ok(keys.iter().filter(|(id, _)| id == "expires-after").map(|_| expiry()).collect())
    });
    repo
}

fn new_template(content: serde_json::Value) -> NewPolicyTemplateDto {
    serde_json::from_value(json!({
        "id": "thirty-days",
        "version": "1",
        "kind": "policy",
        "content": content,
        "parameters": {}
    }))
    .unwrap()
}

/// A template is stored with the versions it references.
#[tokio::test]
async fn template_is_stored_with_its_references() {
    let mut repo = repo_with_expiry();
    repo.expect_insert()
        .withf(|command| command.references == vec![("expires-after".to_string(), "1".to_string())])
        .returning(|command| {
            Ok(piece("thirty-days", command.kind.as_str(), command.content.clone(), json!({}), json!([])))
        });
    let ctx = TestContext::new().template_repo(repo).build();

    let content = json!({"permission": [{"action": "use", "constraint": [
        {"@ref": "expires-after@1", "with": {"$duration": "P30D"}}
    ]}]});
    ctx.templates()
        .create_policy_template(&TestUsers::alone("ana"), &new_template(content))
        .await
        .unwrap();
}

/// A reference to a version the user cannot see is refused, unstored.
#[tokio::test]
async fn unseen_reference_is_refused() {
    let mut repo = MockPolicyTemplateRepoTrait::new();
    repo.expect_get_by_keys().returning(|_, _| Ok(vec![]));
    let ctx = TestContext::new().template_repo(repo).build();

    let content = json!({"permission": [{"action": "use", "constraint": [{"@ref": "secret@1"}]}]});
    let refused = ctx
        .templates()
        .create_policy_template(&TestUsers::alone("ana"), &new_template(content))
        .await
        .unwrap_err();
    assert_eq!(refused.info().status_code.as_u16(), 400);
}

/// A version another one references cannot be deleted.
#[tokio::test]
async fn referenced_version_is_not_deleted() {
    let mut repo = MockPolicyTemplateRepoTrait::new();
    repo.expect_dependents().returning(|_, _| Ok(vec!["thirty-days@1".into()]));
    let ctx = TestContext::new().template_repo(repo).build();

    let refused = ctx
        .templates()
        .delete_policy_template_by_version_and_id(&TestUsers::alone("ana"), "expires-after", "1")
        .await
        .unwrap_err();
    assert_eq!(refused.info().status_code.as_u16(), 409);
}

/// Usages list the template versions and the offers built from a version.
#[tokio::test]
async fn usages_list_templates_and_offers() {
    let mut repo = MockPolicyTemplateRepoTrait::new();
    repo.expect_get_version().returning(|_, _, _| Ok(Some(expiry())));
    repo.expect_dependents().returning(|_, _| Ok(vec!["thirty-days@1".into()]));
    let mut offers = MockOdrlOfferRepoTrait::new();
    offers.expect_using_template().returning(|_, _, _| {
        Ok(vec!["urn:odrl-policy:00000000-0000-0000-0000-000000000001".parse().unwrap()])
    });
    let ctx = TestContext::new().template_repo(repo).offer_repo(offers).build();

    let usages = ctx
        .templates()
        .usages(&TestUsers::alone("ana"), "expires-after", "1")
        .await
        .unwrap();
    assert_eq!(usages.templates, vec!["thirty-days@1"]);
    assert_eq!(usages.offers, vec!["urn:odrl-policy:00000000-0000-0000-0000-000000000001"]);
}

/// A draft previews as plain ODRL with the control points of its pieces, storing nothing.
#[tokio::test]
async fn draft_previews_without_storing() {
    let ctx = TestContext::new().template_repo(repo_with_expiry()).build();
    let request: TemplatePreviewDto = serde_json::from_value(json!({
        "template": {
            "kind": "policy",
            "controlPoints": ["negotiation"],
            "content": {"permission": [{"action": "use", "constraint": [
                {"@ref": "expires-after@1", "with": {"$duration": "$d"}}
            ]}]},
            "parameters": {"$d": {"dataType": "duration"}}
        },
        "parameters": {"$d": "P1Y"}
    }))
    .unwrap();

    let preview = ctx.templates().preview(&TestUsers::alone("ana"), &request).await.unwrap();
    assert_eq!(
        preview.odrl,
        json!({"permission": [{"action": "use", "constraint": [
            {"leftOperand": "odrl:elapsedTime", "operator": "lteq", "rightOperand": "P1Y"}
        ]}]})
    );
    assert_eq!(preview.control_points, vec![ControlPoint::Negotiation, ControlPoint::Validity]);
}
