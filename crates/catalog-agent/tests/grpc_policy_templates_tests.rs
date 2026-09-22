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

//! gRPC adapter tests for the policy-template service: Struct/Value payloads and paging.

mod grpc_fixtures;

use std::collections::HashMap;
use std::sync::Arc;

use catalog_agent::entities::policy_templates::types::LocalizedText;
use catalog_agent::entities::policy_templates::{MockPolicyTemplateEntityTrait, PolicyTemplateDto};
use catalog_agent::grpc::api::catalog_agent::policy_template_entity_service_server::PolicyTemplateEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CreatePolicyTemplateRequest, GetByVersionRequest, ListPolicyTemplatesRequest,
};
use catalog_agent::grpc::policy_templates::PolicyTemplateEntityGrpc;
use chrono::Utc;
use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::paginated_spec::Paginated;
use grpc_fixtures::{owner, request, StubValidator, OTHER_TENANT, TENANT};
use prost_types::value::Kind;
use serde_json::json;
use tonic::Code;

fn grpc(service: MockPolicyTemplateEntityTrait) -> PolicyTemplateEntityGrpc {
    PolicyTemplateEntityGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn dto(version: &str) -> PolicyTemplateDto {
    PolicyTemplateDto {
        id: "tpl".into(),
        tenant_id: TENANT.to_string(),
        version: version.into(),
        date: Utc::now().into(),
        title: Some(LocalizedText::Single("Title".into())),
        description: None,
        author: "me".into(),
        content: serde_json::from_value(json!({"permission": [{"action": "use"}]})).unwrap(),
        parameters: HashMap::new(),
    }
}

fn by_version(id: &str, version: &str) -> GetByVersionRequest {
    GetByVersionRequest {
        id: id.into(),
        version: version.into(),
    }
}

fn valid_create() -> CreatePolicyTemplateRequest {
    CreatePolicyTemplateRequest {
        id: Some("tpl".into()),
        version: Some("1".into()),
        author: Some("me".into()),
        date: Some("2026-01-01T00:00:00Z".into()),
        title: Some(json!("Title").into_prost_value()),
        description: Some(json!([{"@value": "Desc", "@language": "en"}]).into_prost_value()),
        content: Some(json!({"permission": [{"action": "use"}]}).into_prost_struct()),
        parameters: Some(json!({}).into_prost_struct()),
    }
}

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockPolicyTemplateEntityTrait::new());
    let err = g
        .get_policy_template_by_version(request(by_version("tpl", "1"), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockPolicyTemplateEntityTrait::new());
    let err = g
        .get_policy_template_by_version(request(
            by_version("tpl", "1"),
            Some("owner"),
            Some(OTHER_TENANT),
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn create_rejects_missing_content_bad_date_and_malformed_title() {
    let g = grpc(MockPolicyTemplateEntityTrait::new());
    let cases = [
        (
            CreatePolicyTemplateRequest {
                content: None,
                ..valid_create()
            },
            "content: is required",
        ),
        (
            CreatePolicyTemplateRequest {
                date: Some("yesterday".into()),
                ..valid_create()
            },
            "date:",
        ),
        (
            CreatePolicyTemplateRequest {
                title: Some(json!(42).into_prost_value()),
                ..valid_create()
            },
            "title:",
        ),
    ];
    for (req, prefix) in cases {
        let err = g.create_policy_template(owner(req)).await.unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
        assert!(err.message().starts_with(prefix), "{}", err.message());
    }
}

#[tokio::test]
async fn create_maps_localized_text_and_content() {
    let mut svc = MockPolicyTemplateEntityTrait::new();
    svc.expect_create_policy_template()
        .withf(|_, dto| {
            matches!(dto.title, Some(LocalizedText::Single(ref s)) if s == "Title")
                && matches!(dto.description, Some(LocalizedText::Multiple(ref v)) if v.len() == 1)
                && dto.date.is_some()
                && dto.content.permission.as_ref().map(|p| p.len()) == Some(1)
        })
        .returning(|_, _| Ok(dto("1")));
    let g = grpc(svc);
    assert!(g
        .create_policy_template(owner(valid_create()))
        .await
        .is_ok());
}

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockPolicyTemplateEntityTrait::new();
    svc.expect_get_policies_template_by_version_and_id()
        .returning(|_, id, _| Err(ResourceError::not_found(id, "policy template")));
    let g = grpc(svc);
    let err = g
        .get_policy_template_by_version(owner(by_version("tpl", "9")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn list_propagates_paging_and_serializes_struct_fields() {
    let mut svc = MockPolicyTemplateEntityTrait::new();
    svc.expect_get_all_policy_templates()
        .withf(|_, filter, page, _| filter.author.as_deref() == Some("me") && page.limit == 1)
        .returning(|_, _, _, _| Ok(Paginated::new(vec![dto("1")], Some("n".into()), Some(5))));
    let g = grpc(svc);
    let req = ListPolicyTemplatesRequest {
        author: "me".into(),
        limit: 1,
        ..Default::default()
    };
    let resp = g
        .get_all_policy_templates(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.next_cursor, "n");
    assert_eq!(resp.total, 5);
    let item = &resp.items[0];
    assert_eq!(
        item.title.as_ref().unwrap().kind,
        Some(Kind::StringValue("Title".into()))
    );
    assert!(item.description.is_none());
    assert!(item
        .content
        .as_ref()
        .unwrap()
        .fields
        .contains_key("permission"));
    assert!(item.parameters.as_ref().unwrap().fields.is_empty());
}
