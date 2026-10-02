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

//! Placeholders found in HTTP pull and push specs: URL, method, headers and body.

use super::*;

/// A template placeholder in url_template should be found.
#[test]
fn pull_http_url_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/{{__RESOURCE_ID__}}".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("RESOURCE_ID", found[0]);
}

/// Multiple placeholders in the same URL should all be found.
#[test]
fn pull_http_url_with_multiple_parameters_extracts_all() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://{{__HOST__}}/{{__PATH__}}".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(2, found.len());
    assert!(found.contains(&"HOST".to_string()));
    assert!(found.contains(&"PATH".to_string()));
}

/// When method is a Template string (not a Value vec), the placeholder is extracted.
#[test]
fn pull_http_method_as_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Template("{{__METHOD__}}".to_string()),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("METHOD", found[0]);
}

/// A Value vec where one element contains a placeholder should be scanned.
#[test]
fn pull_http_method_value_with_template_item_extracts_parameter() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec![
                    "GET".to_string(),
                    "{{__EXTRA_METHOD__}}".to_string(),
                ]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("EXTRA_METHOD", found[0]);
}

/// The map extractor scans the value of the "__EXTRA__" key for placeholders.
#[test]
fn pull_http_headers_extra_key_extracts_parameter() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: Some(TemplateMapString::Value(HashMap::from([(
                    "__EXTRA__".to_string(),
                    "Bearer {{__TOKEN__}}".to_string(),
                )]))),
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("TOKEN", found[0]);
}

/// Without the "__EXTRA__" key, the map extractor scans nothing.
#[test]
fn pull_http_headers_without_extra_key_extracts_nothing() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: Some(TemplateMapString::Value(HashMap::from([(
                    "Authorization".to_string(),
                    "Bearer {{__TOKEN__}}".to_string(),
                )]))),
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// When headers is a Template string the placeholder is extracted directly.
#[test]
fn pull_http_headers_as_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: Some(TemplateMapString::Template("{{__HEADERS__}}".to_string())),
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("HEADERS", found[0]);
}

/// A placeholder in body_template should be extracted.
#[test]
fn pull_http_body_template_extracts_parameter() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: Some("{{__PAYLOAD__}}".to_string()),
            }),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("PAYLOAD", found[0]);
}

/// When headers and body_template are None and url is literal, nothing is found.
#[test]
fn pull_http_no_optional_fields_extracts_nothing() {
    let dto = ConnectorTemplateDto {
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
                url_template: "https://api.example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        parameters: vec![],
    };
    assert!(run(dto).is_empty());
}

/// With no unsubscribe, only the subscribe spec is scanned.
#[test]
fn push_http_subscribe_only_extracts_parameters() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/{{__RESOURCE_ID__}}/subscribe".to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: None,
            }),
            unsubscribe: None,
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(1, found.len());
    assert_eq!("RESOURCE_ID", found[0]);
}

/// Both subscribe and unsubscribe specs are scanned; all parameters are collected.
#[test]
fn push_http_with_unsubscribe_extracts_parameters_from_both() {
    let dto = ConnectorTemplateDto {
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/{{__SUBSCRIBE_ID__}}/subscribe".to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: None,
            }),
            unsubscribe: Some(ProtocolSpec::Http(HttpSpec {
                url_template: "https://api.example.com/{{__UNSUBSCRIBE_ID__}}/unsubscribe"
                    .to_string(),
                method: TemplateVecString::Value(vec!["DELETE".to_string()]),
                headers: None,
                body_template: None,
            })),
        }),
        parameters: vec![],
    };
    let found = run(dto);
    assert_eq!(2, found.len());
    assert!(found.contains(&"SUBSCRIBE_ID".to_string()));
    assert!(found.contains(&"UNSUBSCRIBE_ID".to_string()));
}
