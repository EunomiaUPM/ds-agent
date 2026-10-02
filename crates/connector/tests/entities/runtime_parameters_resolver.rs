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

//! RuntimeParametersResolver: `__RUNTIME_*__` placeholders filled from the subscribe response
//! with jq, or from the keystore through a fake lookup.

use connector::entities::parameters::keystore_lookup::KeystoreLookup;
use connector::ConnectorInstanceDto;
use std::sync::Arc;

use connector::entities::auth_config::AuthenticationConfig;
use connector::entities::interaction::{InteractionConfig, PushLifecycle};
use connector::entities::parameters::runtime_parameters_resolver::*;
use connector::entities::resource::HttpSpec;
use connector::{ConnectorMetadata, ProtocolSpec, TemplateVecString};
use serde_json::json;
use std::str::FromStr;
use urn::Urn;

fn push_instance(subscribe_url: &str, unsubscribe_url: Option<&str>) -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: Urn::from_str("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap(),
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: subscribe_url.to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: None,
            }),
            unsubscribe: unsubscribe_url.map(|u| {
                ProtocolSpec::Http(HttpSpec {
                    url_template: u.to_string(),
                    method: TemplateVecString::Value(vec!["DELETE".to_string()]),
                    headers: None,
                    body_template: None,
                })
            }),
        }),
        distribution_id: Urn::from_str("urn:uuid:00000000-0000-0000-0000-000000000002").unwrap(),
    }
}

async fn resolve_helper(
    instance: &ConnectorInstanceDto,
    params: serde_json::Value,
) -> ConnectorInstanceDto {
    RuntimeParametersResolver::new(instance, &params)
        .resolve()
        .await
        .unwrap()
}

fn subscribe_url(instance: &ConnectorInstanceDto) -> &str {
    match &instance.interaction {
        InteractionConfig::Push(lc) => match &lc.subscribe {
            ProtocolSpec::Http(s) => &s.url_template,
            _ => panic!("expected HTTP"),
        },
        _ => panic!("expected Push"),
    }
}

/// A jq placeholder inside a URL is replaced by the value it selects.
#[tokio::test]
async fn resolves_interpolated_jq_expression() {
    let instance = push_instance(
        "https://api.example.com/{{__RUNTIME_JSON_{.subscribe.id}__}}/hook",
        None,
    );
    let params = json!({ "subscribe": { "id": "abc-123" } });
    let result = resolve_helper(&instance, params).await;
    assert_eq!(
        subscribe_url(&result),
        "https://api.example.com/abc-123/hook"
    );
}

/// A field that is only a jq placeholder becomes the selected string.
#[tokio::test]
async fn exact_placeholder_resolves_to_string() {
    let instance = push_instance("{{__RUNTIME_JSON_{.subscribe.id}__}}", None);
    let params = json!({ "subscribe": { "id": "resolved-id" } });
    let result = resolve_helper(&instance, params).await;
    assert_eq!(subscribe_url(&result), "resolved-id");
}

/// A field without runtime placeholders is left as is.
#[tokio::test]
async fn leaves_unmatched_string_unchanged() {
    let url = "https://api.example.com/static";
    let instance = push_instance(url, None);
    let result = resolve_helper(&instance, json!({})).await;
    assert_eq!(subscribe_url(&result), url);
}

/// A jq path that selects nothing leaves the placeholder in place.
#[tokio::test]
async fn missing_jq_path_leaves_placeholder_unchanged() {
    let raw = "{{__RUNTIME_JSON_{.subscribe.missing}__}}";
    let instance = push_instance(raw, None);
    let params = json!({ "subscribe": { "id": "x" } });
    let result = resolve_helper(&instance, params).await;
    assert_eq!(subscribe_url(&result), raw);
}

/// Array indexing and nested paths work in jq expressions.
#[tokio::test]
async fn supports_complex_jq_expression() {
    let instance = push_instance("{{__RUNTIME_JSON_{.items[0].name}__}}", None);
    let params = json!({ "items": [{ "name": "first" }, { "name": "second" }] });
    let result = resolve_helper(&instance, params).await;
    assert_eq!(subscribe_url(&result), "first");
}

/// A keystore parameter placeholder is replaced by the stored value.
#[tokio::test]
async fn resolves_parameter_placeholder_exact() {
    struct FakeLookup;
    #[async_trait::async_trait]
    impl KeystoreLookup for FakeLookup {
        async fn get_parameter(&self, _tenant_id: &str, key: &str) -> Option<serde_json::Value> {
            if key == "/my/param" {
                Some(json!("param-value"))
            } else {
                None
            }
        }
        async fn get_secret(&self, _tenant_id: &str, _: &str) -> Option<serde_json::Value> {
            None
        }
    }
    let instance = push_instance("{{__RUNTIME_PARAMETER_{/my/param}__}}", None);
    let result = RuntimeParametersResolver::new(&instance, &json!({}))
        .with_keystore(Arc::new(FakeLookup), "tenant-1")
        .resolve()
        .await
        .unwrap();
    assert_eq!(subscribe_url(&result), "param-value");
}

/// A keystore secret placeholder inside a URL is replaced by the secret.
#[tokio::test]
async fn resolves_secret_placeholder_interpolated() {
    struct FakeLookup;
    #[async_trait::async_trait]
    impl KeystoreLookup for FakeLookup {
        async fn get_parameter(&self, _tenant_id: &str, _: &str) -> Option<serde_json::Value> {
            None
        }
        async fn get_secret(&self, _tenant_id: &str, key: &str) -> Option<serde_json::Value> {
            if key == "/my/secret" {
                Some(json!("s3cr3t"))
            } else {
                None
            }
        }
    }
    let instance = push_instance(
        "https://host/{{__RUNTIME_SECRET_{/my/secret}__}}/path",
        None,
    );
    let result = RuntimeParametersResolver::new(&instance, &json!({}))
        .with_keystore(Arc::new(FakeLookup), "tenant-1")
        .resolve()
        .await
        .unwrap();
    assert_eq!(subscribe_url(&result), "https://host/s3cr3t/path");
}
