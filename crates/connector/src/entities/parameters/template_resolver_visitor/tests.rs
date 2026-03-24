/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use super::*;
use crate::entities::auth_config::BasicAuthConfig;
use crate::entities::auth_config::{ApiKeyLocation, OAuthGrantType};
use crate::entities::common::secret_management::{SecretSource, SecretString};
use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use crate::entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
use crate::entities::parameters::parameters::TemplateMapString as TMS;
use crate::entities::parameters::template_parameters_resolver::TemplateParametersResolver;
use crate::entities::resource::{HttpSpec, KafkaSpec};
use crate::{AuthenticationConfig, ProtocolSpec, TemplateVecString};
use serde_json::json;
use std::collections::HashMap;

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

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

fn resolver(params: &HashMap<String, serde_json::Value>) -> TemplateParametersResolver {
    TemplateParametersResolver::new(params)
}

// -------------------------------------------------------------------------
// HTTP interaction
// -------------------------------------------------------------------------

#[test]
fn resolves_url_template() {
    let mut template = pull_http("https://api.example.com/{{__RESOURCE__}}");
    let params = HashMap::from([("RESOURCE".to_string(), json!("items"))]);

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    let url = match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => s.url_template.clone(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert_eq!(url, "https://api.example.com/items");
}

#[test]
fn resolves_method_template_variant() {
    let mut template = ConnectorTemplateDto {
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

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    let method = match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => s.method.clone(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert!(matches!(method, TemplateVecString::Value(v) if v == vec!["POST"]));
}

#[test]
fn resolves_body_template() {
    let mut template = ConnectorTemplateDto {
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

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    let body = match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => s.body_template.clone().unwrap(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert_eq!(body, r#"{"id":"abc123"}"#);
}

#[test]
fn resolves_headers_map_value_variant() {
    let mut template = ConnectorTemplateDto {
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: Some(TMS::Value(HashMap::from([(
                    "Authorization".to_string(),
                    "Bearer {{__TOKEN__}}".to_string(),
                )]))),
                body_template: None,
            }),
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("TOKEN".to_string(), json!("secret"))]);

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    let headers = match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => s.headers.clone().unwrap(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    match headers {
        TMS::Value(map) => assert_eq!(map["Authorization"], "Bearer secret"),
        _ => panic!("expected Value variant"),
    }
}

// -------------------------------------------------------------------------
// Kafka interaction
// -------------------------------------------------------------------------

#[test]
fn resolves_kafka_topic() {
    let mut template = ConnectorTemplateDto {
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

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    let topic = match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Kafka(s) => s.topic.clone(),
            _ => panic!(),
        },
        _ => panic!(),
    };
    assert_eq!(topic, "events");
}

// -------------------------------------------------------------------------
// Authentication
// -------------------------------------------------------------------------

#[test]
fn resolves_basic_auth_username() {
    let mut template = ConnectorTemplateDto {
        authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
            username: "{{__USERNAME__}}".to_string(),
            password: SecretString {
                source: SecretSource::Plain("pass".to_string()),
            },
        }),
        ..pull_http("https://api.example.com")
    };
    let params = HashMap::from([("USERNAME".to_string(), json!("alice"))]);

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    match &template.authentication {
        AuthenticationConfig::BasicAuth(c) => assert_eq!(c.username, "alice"),
        _ => panic!(),
    }
}

#[test]
fn resolves_api_key_name() {
    let mut template = ConnectorTemplateDto {
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

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    match &template.authentication {
        AuthenticationConfig::ApiKey { key, .. } => assert_eq!(key, "X-Custom-Key"),
        _ => panic!(),
    }
}

#[test]
fn resolves_oauth2_token_url_and_client_id() {
    let mut template = ConnectorTemplateDto {
        authentication: AuthenticationConfig::OAuth2 {
            grant_type: OAuthGrantType::ClientCredentials,
            token_url: "{{__TOKEN_URL__}}".to_string(),
            client_id: "{{__CLIENT_ID__}}".to_string(),
            client_secret: SecretString {
                source: SecretSource::Plain("s3cr3t".to_string()),
            },
            scopes: TemplateVecString::Value(vec![]),
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

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    match &template.authentication {
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

// -------------------------------------------------------------------------
// No-op when no placeholders / unknown params
// -------------------------------------------------------------------------

#[test]
fn leaves_literal_fields_unchanged() {
    let url = "https://api.example.com/data";
    let mut template = pull_http(url);
    let params = HashMap::new();

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => assert_eq!(s.url_template, url),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn leaves_unresolved_placeholder_unchanged_when_param_missing() {
    let mut template = pull_http("https://api.example.com/{{__MISSING__}}");
    let params = HashMap::new();

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    match &template.interaction {
        InteractionConfig::Pull(lc) => match &lc.data_access {
            ProtocolSpec::Http(s) => {
                assert_eq!(s.url_template, "https://api.example.com/{{__MISSING__}}")
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

// -------------------------------------------------------------------------
// Push lifecycle
// -------------------------------------------------------------------------

#[test]
fn resolves_push_subscribe_and_unsubscribe() {
    let mut template = ConnectorTemplateDto {
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

    TemplateResolverVisitor::new(&mut resolver(&params))
        .apply(&mut template)
        .unwrap();

    match &template.interaction {
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
