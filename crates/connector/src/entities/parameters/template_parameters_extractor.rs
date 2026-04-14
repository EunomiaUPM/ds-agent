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

use super::FoundParameter;
use super::{template_parameter_regex, template_sys_parameter_regex};
use super::{FoundParameterType, TemplateMapString, TemplateString};
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use crate::TemplateVecString;
use std::collections::HashMap;
use ymir::errors::Outcome;

pub struct TemplateParametersExtractor {
    found_parameters: Vec<FoundParameter>,
    regex_fn: fn() -> &'static regex::Regex,
}

impl TemplateParametersExtractor {
    pub fn new() -> Self {
        Self {
            found_parameters: Vec::new(),
            regex_fn: template_parameter_regex,
        }
    }

    pub fn just_system_parameters(mut self) -> Self {
        self.regex_fn = template_sys_parameter_regex;
        self
    }

    pub fn found_parameters(&self) -> &[FoundParameter] {
        &self.found_parameters
    }

    fn scan_str(&mut self, value: &str, content_type: &FoundParameterType) {
        let re = (self.regex_fn)();
        for cap in re.captures_iter(value) {
            self.found_parameters.push(FoundParameter {
                name: cap[1].to_string(),
                content_type: *content_type,
            });
        }
    }

    fn vec_string_parameter_extractor(&mut self, template: &[String]) {
        for s in template {
            self.scan_str(s, &FoundParameterType::VecString);
        }
    }
    fn map_string_parameter_extractor(&mut self, template: &HashMap<String, String>) {
        if let Some(extra) = template.get("__EXTRA__") {
            self.scan_str(extra, &FoundParameterType::MapString);
        }
    }
}

impl ConnectorTemplateWalker for TemplateParametersExtractor {
    fn on_string(&mut self, field: &mut TemplateString) -> Outcome<()> {
        self.scan_str(field, &FoundParameterType::String);
        Ok(())
    }

    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()> {
        match field {
            TemplateVecString::Template(t) => self.scan_str(t, &FoundParameterType::VecString),
            TemplateVecString::Value(v) => self.vec_string_parameter_extractor(v),
        };
        Ok(())
    }

    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()> {
        match field {
            TemplateMapString::Template(t) => self.scan_str(t, &FoundParameterType::MapString),
            TemplateMapString::Value(v) => self.map_string_parameter_extractor(v),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::auth_config::{ApiKeyLocation, BasicAuthConfig, OAuthGrantType};
    use crate::entities::common::secret_management::{SecretSource, SecretString};
    use crate::entities::connector_template::ConnectorTemplateDto;
    use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
    use crate::entities::parameters::template_parameters_extractor::TemplateParametersExtractor;
    use crate::entities::parameters::{FoundParameterType, TemplateMapString};
    use crate::entities::resource::KafkaSpec;
    use crate::{
        AuthenticationConfig, ConnectorMetadata, HttpSpec, InteractionConfig, ProtocolSpec,
        PullLifecycle, PushLifecycle, TemplateVecString,
    };
    use std::collections::HashMap;

    // =========================================================================
    // Helpers
    // =========================================================================

    fn run(mut dto: ConnectorTemplateDto) -> Vec<String> {
        let mut extractor = TemplateParametersExtractor::new();
        extractor.walk(&mut dto).expect("TODO: panic message");
        extractor
            .found_parameters()
            .iter()
            .map(|fp| fp.name.clone())
            .collect()
    }

    // =========================================================================
    // Extract only fields
    // =========================================================================

    #[test]
    fn complete_value_extractor_on_string() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field: String = "{{__TEST__}}".to_string();
        extractor.on_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0].name, "TEST");
        assert_eq!(found_parameters[0].content_type, FoundParameterType::String);
    }
    #[test]
    fn partial_value_extractor_on_string() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field: String = "http://mi-api.com/{{__TEST__}}".to_string();
        extractor.on_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0].name, "TEST");
        assert_eq!(found_parameters[0].content_type, FoundParameterType::String);
    }
    #[test]
    fn nothing_to_extract_on_string() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field: String = "http://mi-api.com/no-test".to_string();
        extractor.on_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 0);
    }
    #[test]
    fn vec_string_parameter_extractor_as_complete() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field = TemplateVecString::Template("{{__TEST__}}".to_string());
        extractor.on_vec_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0].name, "TEST");
        assert_eq!(
            found_parameters[0].content_type,
            FoundParameterType::VecString
        );
    }
    #[test]
    fn vec_string_parameter_extractor_as_partial() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field =
            TemplateVecString::Value(vec!["test".to_string(), "{{__TEST__}}".to_string()]);
        extractor.on_vec_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0].name, "TEST");
        assert_eq!(
            found_parameters[0].content_type,
            FoundParameterType::VecString
        );
    }
    #[test]
    fn nothing_to_extract_on_vec_string() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field = TemplateVecString::Value(vec!["test".to_string(), "test".to_string()]);
        extractor.on_vec_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 0);
    }
    #[test]
    fn map_string_parameter_extractor_as_complete() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field = TemplateMapString::Template("{{__TEST__}}".to_string());
        extractor.on_map_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0].name, "TEST");
        assert_eq!(
            found_parameters[0].content_type,
            FoundParameterType::MapString
        );
    }
    #[test]
    fn map_string_parameter_extractor_as_partial() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field = TemplateMapString::Value(HashMap::from([(
            "__EXTRA__".to_string(),
            "{{__TEST__}}".to_string(),
        )]));
        extractor.on_map_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 1);
        assert_eq!(found_parameters[0].name, "TEST");
        assert_eq!(
            found_parameters[0].content_type,
            FoundParameterType::MapString
        );
    }
    #[test]
    fn nothing_to_extract_on_map_string() {
        let mut extractor = TemplateParametersExtractor::new();
        let mut field =
            TemplateMapString::Value(HashMap::from([("test".to_string(), "test".to_string())]));
        extractor.on_map_string(&mut field).unwrap();
        let found_parameters = extractor.found_parameters();
        assert_eq!(found_parameters.len(), 0);
    }

    // =========================================================================
    // Extract ConnectorTemplateDtos
    // =========================================================================

    fn no_auth_extracts_no_parameters() {
        // NoAuth has no fields to scan, the auth step should contribute nothing.
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

    #[test]
    fn basic_auth_username_template_extracts_parameter() {
        // The visitor scans the username field of BasicAuth.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
                username: "{{__USERNAME__}}".to_string(),
                password: SecretString {
                    source: SecretSource::Plain("secret".to_string()),
                },
            }),
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
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("USERNAME", found[0]);
    }

    #[test]
    fn basic_auth_literal_username_extracts_no_parameters() {
        // A literal username value contains no template placeholders.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
                username: "admin".to_string(),
                password: SecretString {
                    source: SecretSource::Plain("secret".to_string()),
                },
            }),
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

    #[test]
    fn bearer_token_extracts_no_parameters() {
        // The BearerToken arm in the walker is currently a no-op.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::BearerToken {
                token: SecretString {
                    source: SecretSource::Plain("{{__TOKEN__}}".to_string()),
                },
            },
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

    #[test]
    fn api_key_with_literal_key_extracts_no_parameters() {
        // A literal key name has no placeholders; value is SecretString (not scanned).
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::ApiKey {
                key: "X-Api-Key".to_string(),
                value: SecretString {
                    source: SecretSource::Plain("s3cr3t".to_string()),
                },
                location: ApiKeyLocation::Header,
            },
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

    #[test]
    fn api_key_with_template_key_extracts_parameter() {
        // The header/query-param name may itself be parameterised.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::ApiKey {
                key: "{{__API_KEY_HEADER__}}".to_string(),
                value: SecretString {
                    source: SecretSource::Plain("s3cr3t".to_string()),
                },
                location: ApiKeyLocation::Header,
            },
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
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("API_KEY_HEADER", found[0]);
    }

    #[test]
    fn oauth2_with_literal_fields_extracts_no_parameters() {
        // All TemplateString/TemplateVecString fields are literals; client_secret
        // is a SecretString and is intentionally not scanned.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::OAuth2 {
                grant_type: OAuthGrantType::ClientCredentials,
                token_url: "https://auth.example.com/token".to_string(),
                client_id: "my-client".to_string(),
                client_secret: SecretString {
                    source: SecretSource::Plain("s3cr3t".to_string()),
                },
                scopes: TemplateVecString::Value(vec!["read".to_string()]),
            },
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

    #[test]
    fn oauth2_with_template_fields_extracts_parameters() {
        // token_url, client_id, and scopes all support placeholders.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::OAuth2 {
                grant_type: OAuthGrantType::ClientCredentials,
                token_url: "{{__TOKEN_URL__}}".to_string(),
                client_id: "{{__CLIENT_ID__}}".to_string(),
                client_secret: SecretString {
                    source: SecretSource::Plain("s3cr3t".to_string()),
                },
                scopes: TemplateVecString::Template("{{__SCOPES__}}".to_string()),
            },
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
        let found = run(dto);
        assert_eq!(3, found.len());
        assert!(found.contains(&"TOKEN_URL".to_string()));
        assert!(found.contains(&"CLIENT_ID".to_string()));
        assert!(found.contains(&"SCOPES".to_string()));
    }

    // =========================================================================
    // Pull + HTTP
    // =========================================================================

    #[test]
    fn pull_http_url_template_extracts_parameter() {
        // A template placeholder in url_template should be found.
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

    #[test]
    fn pull_http_url_with_multiple_parameters_extracts_all() {
        // Multiple placeholders in the same URL should all be found.
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

    #[test]
    fn pull_http_method_as_template_extracts_parameter() {
        // When method is a Template string (not a Value vec), the placeholder is extracted.
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

    #[test]
    fn pull_http_method_value_with_template_item_extracts_parameter() {
        // A Value vec where one element contains a placeholder should be scanned.
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

    #[test]
    fn pull_http_headers_extra_key_extracts_parameter() {
        // The map extractor scans the value of the "__EXTRA__" key for placeholders.
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

    #[test]
    fn pull_http_headers_without_extra_key_extracts_nothing() {
        // Without the "__EXTRA__" key, the map extractor scans nothing.
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

    #[test]
    fn pull_http_headers_as_template_extracts_parameter() {
        // When headers is a Template string the placeholder is extracted directly.
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

    #[test]
    fn pull_http_body_template_extracts_parameter() {
        // A placeholder in body_template should be extracted.
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

    #[test]
    fn pull_http_no_optional_fields_extracts_nothing() {
        // When headers and body_template are None and url is literal, nothing is found.
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

    // =========================================================================
    // Pull + Kafka
    // =========================================================================

    #[test]
    fn pull_kafka_topic_template_extracts_parameter() {
        // A placeholder in the topic field should be extracted.
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
                data_access: ProtocolSpec::Kafka(KafkaSpec {
                    brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                    topic: "{{__TOPIC__}}".to_string(),
                    group_id: None,
                }),
            }),
            parameters: vec![],
        };
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("TOPIC", found[0]);
    }

    #[test]
    fn pull_kafka_brokers_as_template_extracts_parameter() {
        // When brokers is a Template string, the placeholder is extracted.
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
                data_access: ProtocolSpec::Kafka(KafkaSpec {
                    brokers: TemplateVecString::Template("{{__BROKERS__}}".to_string()),
                    topic: "events".to_string(),
                    group_id: None,
                }),
            }),
            parameters: vec![],
        };
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("BROKERS", found[0]);
    }

    #[test]
    fn pull_kafka_brokers_value_with_template_item_extracts_parameter() {
        // A Value vec where one broker address contains a placeholder should be scanned.
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
                data_access: ProtocolSpec::Kafka(KafkaSpec {
                    brokers: TemplateVecString::Value(vec![
                        "localhost:9092".to_string(),
                        "{{__BROKER_HOST__}}:9092".to_string(),
                    ]),
                    topic: "events".to_string(),
                    group_id: None,
                }),
            }),
            parameters: vec![],
        };
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("BROKER_HOST", found[0]);
    }

    #[test]
    fn pull_kafka_group_id_extracts_parameter() {
        // A placeholder in the optional group_id should be extracted.
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
                data_access: ProtocolSpec::Kafka(KafkaSpec {
                    brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                    topic: "events".to_string(),
                    group_id: Some("{{__GROUP_ID__}}".to_string()),
                }),
            }),
            parameters: vec![],
        };
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("GROUP_ID", found[0]);
    }

    #[test]
    fn pull_kafka_without_group_id_extracts_nothing() {
        // When group_id is None and all other fields are literals, nothing is found.
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
                data_access: ProtocolSpec::Kafka(KafkaSpec {
                    brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                    topic: "events".to_string(),
                    group_id: None,
                }),
            }),
            parameters: vec![],
        };
        assert!(run(dto).is_empty());
    }

    // =========================================================================
    // Push lifecycle
    // =========================================================================

    #[test]
    fn push_http_subscribe_only_extracts_parameters() {
        // With no unsubscribe, only the subscribe spec is scanned.
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
                    url_template: "https://api.example.com/{{__RESOURCE_ID__}}/subscribe"
                        .to_string(),
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

    #[test]
    fn push_http_with_unsubscribe_extracts_parameters_from_both() {
        // Both subscribe and unsubscribe specs are scanned; all parameters are collected.
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
                    url_template: "https://api.example.com/{{__SUBSCRIBE_ID__}}/subscribe"
                        .to_string(),
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

    #[test]
    fn push_kafka_subscribe_without_unsubscribe_extracts_parameters() {
        // Push interaction can also use Kafka; the topic placeholder should be found.
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
                subscribe: ProtocolSpec::Kafka(KafkaSpec {
                    brokers: TemplateVecString::Value(vec!["localhost:9092".to_string()]),
                    topic: "{{__TOPIC__}}".to_string(),
                    group_id: None,
                }),
                unsubscribe: None,
            }),
            parameters: vec![],
        };
        let found = run(dto);
        assert_eq!(1, found.len());
        assert_eq!("TOPIC", found[0]);
    }

    // =========================================================================
    // combined auth + interaction
    // =========================================================================

    #[test]
    fn basic_auth_and_http_url_extracts_all_parameters() {
        // Auth is visited before interaction; parameters appear in that order.
        let dto = ConnectorTemplateDto {
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
                username: "{{__USERNAME__}}".to_string(),
                password: SecretString {
                    source: SecretSource::Plain("secret".to_string()),
                },
            }),
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
        assert_eq!(2, found.len());
        assert_eq!("USERNAME", found[0]); // auth is visited first
        assert_eq!("RESOURCE_ID", found[1]);
    }
}
