use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::{TemplateMapString, TemplateString};
use crate::entities::parameters_ok::connector_template_walker::ConnectorTemplateWalker;
use crate::entities::parameters_ok::template_parameter_regex;
use crate::TemplateVecString;
use regex::Regex;
use std::collections::HashMap;
use ymir::errors::{Errors, Outcome};

pub struct InstanceParametersResolver<'a> {
    connector_template: &'a ConnectorTemplateDto,
    params: &'a HashMap<String, serde_json::Value>,
}

impl<'a> InstanceParametersResolver<'a> {
    pub fn new(
        connector_template: &'a ConnectorTemplateDto,
        params: &'a HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            connector_template,
            params,
        }
    }

    pub fn resolve(&mut self) -> Outcome<ConnectorTemplateDto> {
        let mut template = self.connector_template.clone();
        self.walk(&mut template)?;
        Ok(template)
    }

    fn resolve_key(&self, key: &str) -> Option<serde_json::Value> {
        self.params.get(key).cloned()
    }

    fn try_exact_replacement(&self, re: &Regex, raw: &str) -> Option<serde_json::Value> {
        let caps = re.captures(raw)?;
        let full_match = caps.get(0)?.as_str();
        if full_match != raw {
            return None;
        }
        let key = caps.get(1)?.as_str();
        self.resolve_key(key)
    }

    fn try_string_interpolation(&self, re: &Regex, raw: &str) -> Option<serde_json::Value> {
        if !re.is_match(raw) {
            return None;
        }
        let mut new_string = raw.to_string();
        for caps in re.captures_iter(raw) {
            let full_match = &caps[0];
            let key = &caps[1];

            if let Some(val) = self.resolve_key(key) {
                let replacement_str = self.value_to_string(&val);
                new_string = new_string.replace(full_match, &replacement_str);
            }
        }
        Some(serde_json::Value::String(new_string))
    }

    fn value_to_string(&self, val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::String(s) => s.clone(),
            _ => val.to_string(),
        }
    }

    fn value_resolver(&self, raw: &str) -> Option<serde_json::Value> {
        let re = template_parameter_regex();
        if let Some(val) = self.try_exact_replacement(re, raw) {
            return Some(val);
        }
        self.try_string_interpolation(re, raw)
    }
}

impl<'a> ConnectorTemplateWalker for InstanceParametersResolver<'a> {
    fn on_string(&mut self, field: &mut TemplateString) -> Outcome<()> {
        if let Some(val) = self.value_resolver(field.as_str()) {
            *field = match val {
                serde_json::Value::String(s) => s,
                v => v.to_string(),
            };
        }
        Ok(())
    }

    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()> {
        match field {
            TemplateVecString::Template(tmpl) => {
                if let Some(val) = self.value_resolver(tmpl.as_str()) {
                    let list: Vec<String> = serde_json::from_value(val).map_err(|e| {
                        Errors::crazy(
                            format!("Failed to resolve TemplateVecString for '{}': {}", tmpl, e),
                            None,
                        )
                    })?;
                    *field = TemplateVecString::Value(list);
                }
            }
            TemplateVecString::Value(list) => {
                let mut new_list = Vec::with_capacity(list.len());
                for item in list.iter() {
                    let mut resolved_item = item.clone();
                    if let Some(val) = self.value_resolver(item.as_str()) {
                        match val {
                            serde_json::Value::Array(arr) => {
                                for v in arr {
                                    new_list.push(match v {
                                        serde_json::Value::String(s) => s,
                                        other => other.to_string(),
                                    });
                                }
                                continue;
                            }
                            serde_json::Value::String(s) => resolved_item = s,
                            other => resolved_item = other.to_string(),
                        }
                    }
                    new_list.push(resolved_item);
                }
                *list = new_list;
            }
        }
        Ok(())
    }

    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()> {
        match field {
            TemplateMapString::Template(tmpl) => {
                if let Some(val) = self.value_resolver(tmpl.as_str()) {
                    let map: std::collections::HashMap<String, String> =
                        serde_json::from_value(val).map_err(|e| {
                            Errors::crazy(
                                format!(
                                    "Failed to resolve TemplateMapString for '{}': {}",
                                    tmpl, e
                                ),
                                None,
                            )
                        })?;
                    *field = TemplateMapString::Value(map);
                }
            }
            TemplateMapString::Value(map) => {
                let mut to_merge = std::collections::HashMap::new();
                let mut to_remove = Vec::new();

                for (key, value) in map.iter_mut() {
                    if key == "__EXTRA__" {
                        if let Some(val) = self.value_resolver(value.as_str()) {
                            if let serde_json::Value::Object(obj) = val {
                                for (k, v) in obj {
                                    to_merge.insert(
                                        k,
                                        match v {
                                            serde_json::Value::String(s) => s,
                                            other => other.to_string(),
                                        },
                                    );
                                }
                                to_remove.push(key.clone());
                            } else {
                                *value = match val {
                                    serde_json::Value::String(s) => s,
                                    other => other.to_string(),
                                };
                            }
                        }
                    } else {
                        self.on_string(value)?;
                    }
                }

                for k in to_remove {
                    map.remove(&k);
                }
                for (k, v) in to_merge {
                    map.insert(k, v);
                }
            }
        }
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::auth_config::BasicAuthConfig;
    use crate::entities::auth_config::{ApiKeyLocation, OAuthGrantType};
    use crate::entities::common::secret_management::{SecretSource, SecretString};
    use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
    use crate::entities::interaction::{InteractionConfig, PullLifecycle, PushLifecycle};
    use crate::entities::parameters::TemplateMapString;
    use crate::entities::resource::{HttpSpec, KafkaSpec};
    use crate::{AuthenticationConfig, ProtocolSpec, TemplateVecString};
    use serde_json::json;
    use std::collections::HashMap;

    // =========================================================================
    // Helpers
    // =========================================================================

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

    // =========================================================================
    // HTTP interaction
    // =========================================================================

    #[test]
    fn resolves_url_template() {
        let template = pull_http("https://api.example.com/{{__RESOURCE__}}");
        let params = HashMap::from([("RESOURCE".to_string(), json!("items"))]);
        let resolved = resolve(&template, params);
        assert_eq!(http_spec(&resolved).url_template, "https://api.example.com/items");
    }

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

    // =========================================================================
    // Kafka interaction
    // =========================================================================

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

    // =========================================================================
    // Authentication
    // =========================================================================

    #[test]
    fn resolves_basic_auth_username() {
        let template = ConnectorTemplateDto {
            authentication: AuthenticationConfig::BasicAuth(BasicAuthConfig {
                username: "{{__USERNAME__}}".to_string(),
                password: SecretString { source: SecretSource::Plain("pass".to_string()) },
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

    #[test]
    fn resolves_api_key_name() {
        let template = ConnectorTemplateDto {
            authentication: AuthenticationConfig::ApiKey {
                key: "{{__HEADER_NAME__}}".to_string(),
                value: SecretString { source: SecretSource::Plain("s3cr3t".to_string()) },
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

    #[test]
    fn resolves_oauth2_token_url_and_client_id() {
        let template = ConnectorTemplateDto {
            authentication: AuthenticationConfig::OAuth2 {
                grant_type: OAuthGrantType::ClientCredentials,
                token_url: "{{__TOKEN_URL__}}".to_string(),
                client_id: "{{__CLIENT_ID__}}".to_string(),
                client_secret: SecretString { source: SecretSource::Plain("s3cr3t".to_string()) },
                scopes: TemplateVecString::Value(vec![]),
            },
            ..pull_http("https://api.example.com")
        };
        let params = HashMap::from([
            ("TOKEN_URL".to_string(), json!("https://auth.example.com/token")),
            ("CLIENT_ID".to_string(), json!("my-client")),
        ]);
        let resolved = resolve(&template, params);
        match &resolved.authentication {
            AuthenticationConfig::OAuth2 { token_url, client_id, .. } => {
                assert_eq!(token_url, "https://auth.example.com/token");
                assert_eq!(client_id, "my-client");
            }
            _ => panic!(),
        }
    }

    // =========================================================================
    // No-op cases
    // =========================================================================

    #[test]
    fn leaves_literal_fields_unchanged() {
        let url = "https://api.example.com/data";
        let template = pull_http(url);
        let resolved = resolve(&template, HashMap::new());
        assert_eq!(http_spec(&resolved).url_template, url);
    }

    #[test]
    fn leaves_unresolved_placeholder_unchanged_when_param_missing() {
        let template = pull_http("https://api.example.com/{{__MISSING__}}");
        let resolved = resolve(&template, HashMap::new());
        assert_eq!(
            http_spec(&resolved).url_template,
            "https://api.example.com/{{__MISSING__}}"
        );
    }

    // =========================================================================
    // Push lifecycle
    // =========================================================================

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
                    ProtocolSpec::Http(s) => assert_eq!(
                        s.url_template,
                        "https://api.example.com/res-42/subscribe"
                    ),
                    _ => panic!(),
                }
                match lc.unsubscribe.as_ref().unwrap() {
                    ProtocolSpec::Http(s) => assert_eq!(
                        s.url_template,
                        "https://api.example.com/res-42/unsubscribe"
                    ),
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }
}
