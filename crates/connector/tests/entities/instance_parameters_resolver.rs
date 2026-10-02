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

//! InstanceParametersResolver: placeholders filled from instance parameters across the
//! template's protocol and auth fields.

use connector::entities::connector_template::ConnectorMetadata;
use connector::entities::connector_template::ConnectorTemplateDto;
use connector::entities::parameters::{TemplateMapString, TemplateVecString};
use std::collections::HashMap;

use connector::entities::auth_config::BasicAuthConfig;
use connector::entities::auth_config::{ApiKeyLocation, OAuthGrantType};
use connector::entities::common::secret_management::{SecretSource, SecretString};
use connector::entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
use connector::entities::parameters::instance_parameters_resolver::*;
use connector::entities::resource::{HttpSpec, KafkaSpec};
use connector::{AuthenticationConfig, ProtocolSpec};
use serde_json::json;

fn pull_http(url: &str) -> ConnectorTemplateDto {
    ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: url.to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    }
}

fn resolve(
    template: &ConnectorTemplateDto,
    params: HashMap<String, serde_json::Value>,
) -> ConnectorTemplateDto {
    InstanceParametersResolver::new(template, &params)
        .resolve()
        .unwrap()
}

fn http_spec(template: &ConnectorTemplateDto) -> &HttpSpec {
    match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => s,
            _ => panic!("expected Http"),
        },
        _ => panic!("expected Pull"),
    }
}

/// A placeholder in the URL is replaced by its parameter.
#[test]
fn resolves_url_template() {
    let template = pull_http("https://api.example.com/{{__RESOURCE__}}");
    let params = HashMap::from([("RESOURCE".to_string(), json!("items"))]);
    let resolved = resolve(&template, params);
    assert_eq!(
        http_spec(&resolved).url_template,
        "https://api.example.com/items"
    );
}

/// A templated method becomes the parameter's list value.
#[test]
fn resolves_method_template_variant() {
    let template = ConnectorTemplateDto {
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Template("{{__METHOD__}}".to_string()),
                headers: None,
                body_template: None,
            }),
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("METHOD".to_string(), json!(["POST"]))]);
    let resolved = resolve(&template, params);
    assert!(
        matches!(&http_spec(&resolved).method, TemplateVecString::Value(v) if v == &vec!["POST"])
    );
}

/// A placeholder in the body template is replaced.
#[test]
fn resolves_body_template() {
    let template = ConnectorTemplateDto {
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: Some(r#"{"id":"{{__ID__}}"}"#.to_string()),
            }),
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("ID".to_string(), json!("abc123"))]);
    let resolved = resolve(&template, params);
    assert_eq!(
        http_spec(&resolved).body_template.as_deref().unwrap(),
        r#"{"id":"abc123"}"#
    );
}

/// Placeholders inside header values are replaced.
#[test]
fn resolves_headers_map_value_variant() {
    let template = ConnectorTemplateDto {
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: Some(TemplateMapString::Value(HashMap::from([(
                    "Authorization".to_string(),
                    "Bearer {{__TOKEN__}}".to_string(),
                )]))),
                body_template: None,
            }),
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("TOKEN".to_string(), json!("secret"))]);
    let resolved = resolve(&template, params);
    match http_spec(&resolved).headers.as_ref().unwrap() {
        TemplateMapString::Value(map) => assert_eq!(map["Authorization"], "Bearer secret"),
        _ => panic!("expected Value variant"),
    }
}

/// A placeholder in a Kafka topic is replaced.
#[test]
fn resolves_kafka_topic() {
    let template = ConnectorTemplateDto {
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                topic: "{{__TOPIC__}}".to_string(),
                group_id: None,
            }),
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("TOPIC".to_string(), json!("events"))]);
    let resolved = resolve(&template, params);
    match &resolved.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Kafka(s) => assert_eq!(s.topic, "events"),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

/// A placeholder in the Basic auth username is replaced.
#[test]
fn resolves_basic_auth_username() {
    let template = ConnectorTemplateDto {
        authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
            username: "{{__USERNAME__}}".to_string(),
            password: SecretString {
                source: SecretSource::Plain("pass".to_string()),
            },
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("USERNAME".to_string(), json!("alice"))]);
    let resolved = resolve(&template, params);
    match &resolved.authentication {
        AuthenticationConfig::BasicAuth(c) => assert_eq!(c.username, "alice"),
        _ => panic!(),
    }
}

/// A placeholder in the API key name is replaced.
#[test]
fn resolves_api_key_name() {
    let template = ConnectorTemplateDto {
        authentication: AuthenticationConfig::ApiKey {
            key: "{{__HEADER_NAME__}}".to_string(),
            value: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            location: ApiKeyLocation::Header,
        },
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("HEADER_NAME".to_string(), json!("X-Custom-Key"))]);
    let resolved = resolve(&template, params);
    match &resolved.authentication {
        AuthenticationConfig::ApiKey { key, .. } => assert_eq!(key, "X-Custom-Key"),
        _ => panic!(),
    }
}

/// Placeholders in the OAuth2 token URL and client id are replaced.
#[test]
fn resolves_oauth2_token_url_and_client_id() {
    let template = ConnectorTemplateDto {
        authentication: AuthenticationConfig::OAuth2 {
            grant_type: OAuthGrantType::ClientCredentials,
            token_url: "{{__TOKEN_URL__}}".to_string(),
            client_id: "{{__CLIENT_ID__}}".to_string(),
            client_secret: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            scopes: TemplateVecString::Value(vec![]),
            on_token_expire: Default::default(),
        },
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([
        (
            "TOKEN_URL".to_string(),
            json!("https://auth.example.com/token"),
        ),
        ("CLIENT_ID".to_string(), json!("my-client")),
    ]);
    let resolved = resolve(&template, params);
    match &resolved.authentication {
        AuthenticationConfig::OAuth2 {
            token_url,
            client_id,
            ..
        } => {
            assert_eq!(token_url, "https://auth.example.com/token");
            assert_eq!(client_id, "my-client");
        }
        _ => panic!(),
    }
}

/// A field without placeholders is left as is.
#[test]
fn leaves_literal_fields_unchanged() {
    let url = "https://api.example.com/data";
    let template = pull_http(url);
    let resolved = resolve(&template, HashMap::new());
    assert_eq!(http_spec(&resolved).url_template, url);
}

/// A placeholder without parameter is left in place.
#[test]
fn leaves_unresolved_placeholder_unchanged_when_param_missing() {
    let template = pull_http("https://api.example.com/{{__MISSING__}}");
    let resolved = resolve(&template, HashMap::new());
    assert_eq!(
        http_spec(&resolved).url_template,
        "https://api.example.com/{{__MISSING__}}"
    );
}

/// Placeholders in both push subscribe and unsubscribe specs are replaced.
#[test]
fn resolves_push_subscribe_and_unsubscribe() {
    let template = ConnectorTemplateDto {
        interaction: InteractionConfig::Push(PushLifecycle {
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/{{__ID__}}/subscribe".to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: None,
            }),
            unsubscribe: Some(ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/{{__ID__}}/unsubscribe".to_string(),
                method: TemplateVecString::Value(vec!["DELETE".to_string()]),
                headers: None,
                body_template: None,
            })),
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("ID".to_string(), json!("res-42"))]);
    let resolved = resolve(&template, params);
    match &resolved.interaction {
        InteractionConfig::Push(lc) => {
            match &lc.subscribe {
                ProtocolSpec::Http(s) => {
                    assert_eq!(s.url_template, "https://api.example.com/res-42/subscribe")
                }
                _ => panic!(),
            }
            match lc.unsubscribe.as_ref().unwrap() {
                ProtocolSpec::Http(s) => {
                    assert_eq!(s.url_template, "https://api.example.com/res-42/unsubscribe")
                }
                _ => panic!(),
            }
        }
        _ => panic!(),
    }
}
