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

use super::{FoundParameter, ParameterDefinition, ParameterType, SysParameterType};
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::template_parameters_extractor::TemplateParametersExtractor;
use crate::ConnectorInstanceDto;
use serde_json::json;
use std::collections::HashMap;
use ymir::errors::{Errors, Outcome};

pub(crate) struct InstanceParametersMap {
    inner: HashMap<String, serde_json::Value>,
}

impl InstanceParametersMap {
    pub fn collect(&self) -> HashMap<String, serde_json::Value> {
        self.inner.clone()
    }
}

pub struct InstanceParametersMapBuilder {
    own_url: String,
    params: HashMap<String, serde_json::Value>,
}

impl InstanceParametersMapBuilder {
    pub fn new(own_url: String) -> Self {
        Self {
            own_url,
            params: HashMap::new(),
        }
    }

    pub fn with_instance_parameters(
        mut self,
        instance_parameters: &HashMap<String, serde_json::Value>,
    ) -> Self {
        for (k, v) in instance_parameters {
            self.params.entry(k.clone()).or_insert_with(|| v.clone());
        }
        self
    }
    pub fn with_default_parameters(mut self, parameters: &[ParameterDefinition]) -> Outcome<Self> {
        for def in parameters {
            if self.params.contains_key(&def.name) {
                continue;
            }
            if let Some(default_str) = &def.default_value {
                let json_value = Self::cast_default_value(&def.name, &def.param_type, default_str)?;
                self.params.insert(def.name.clone(), json_value);
            }
        }
        Ok(self)
    }

    pub fn with_system_parameters(
        mut self,
        connector_template: &mut ConnectorTemplateDto,
    ) -> Outcome<Self> {
        let mut extractor = TemplateParametersExtractor::new().just_system_parameters();
        extractor.walk(connector_template)?;
        let parameters_found = extractor.found_parameters();
        for found in parameters_found {
            let value = self.resolve_sys_value(&found)?;
            self.params.entry(found.name.clone()).or_insert(value);
        }
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn with_runtime_parameters(
        self,
        _connector_instance: &ConnectorInstanceDto,
    ) -> Outcome<Self> {
        Ok(self)
    }

    pub fn build(self) -> InstanceParametersMap {
        InstanceParametersMap { inner: self.params }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    fn cast_default_value(
        param_name: &str,
        param_type: &ParameterType,
        raw_val: &str,
    ) -> Outcome<serde_json::Value> {
        match param_type {
            ParameterType::String => Ok(json!(raw_val)),
            ParameterType::Int => {
                let parsed = raw_val.parse::<i64>().map_err(|_| {
                    Errors::crazy(
                        format!(
                            "Template Definition Error: Default value '{}' for param '{}' is not a valid Integer.",
                            raw_val, param_name
                        ),
                        None,
                    )
                })?;
                Ok(json!(parsed))
            }
            ParameterType::Boolean => {
                let parsed = raw_val.to_lowercase().parse::<bool>().map_err(|_| {
                    Errors::crazy(
                        format!(
                            "Template Definition Error: Default value '{}' for param '{}' is not a valid Boolean.",
                            raw_val, param_name
                        ),
                        None,
                    )
                })?;
                Ok(json!(parsed))
            }
            ParameterType::VecString | ParameterType::MapStringString => {
                let parsed: serde_json::Value = serde_json::from_str(raw_val).map_err(|e| {
                    Errors::crazy(format!("Template Definition Error: Default value '{}' for complex param '{}' is not valid JSON: {}", raw_val, param_name, e), None)
                })?;

                if matches!(param_type, ParameterType::VecString) && !parsed.is_array() {
                    return Err(Errors::crazy(
                        format!("Default value for '{}' must be a JSON Array", param_name),
                        None,
                    ));
                }
                if matches!(param_type, ParameterType::MapStringString) && !parsed.is_object() {
                    return Err(Errors::crazy(
                        format!("Default value for '{}' must be a JSON Object", param_name),
                        None,
                    ));
                }

                Ok(parsed)
            }
        }
    }

    fn resolve_sys_value(&self, found_parameter: &FoundParameter) -> Outcome<serde_json::Value> {
        let sys_parameter_parsed = found_parameter.name.parse::<SysParameterType>()?;
        match sys_parameter_parsed {
            SysParameterType::SysUrn => {
                let nss = uuid::Uuid::new_v4().to_string();
                let urn = urn::UrnBuilder::new("uuid", &nss).build()?;
                Ok(json!(urn.to_string()))
            }
            SysParameterType::SysToken => Ok(json!(uuid::Uuid::new_v4().to_string())),
            SysParameterType::SysTimestamp => Ok(json!(chrono::Utc::now().timestamp())),
            SysParameterType::SysIso8601 => Ok(json!(chrono::Utc::now().to_rfc3339())),
            SysParameterType::SysOwnUrl {
                host_docker_internal: false,
            } => Ok(json!(self.own_url)),
            SysParameterType::SysOwnUrl {
                host_docker_internal: true,
            } => {
                let docker_url = self
                    .own_url
                    .replace("localhost", "host.docker.internal")
                    .replace("127.0.0.1", "host.docker.internal");
                Ok(json!(docker_url))
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::TemplateVecString;
    use super::*;
    use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
    use crate::entities::interaction::{InteractionConfig, PullLifecycle};
    use crate::entities::resource::HttpSpec;
    use crate::{AuthenticationConfig, ProtocolSpec};
    use serde_json::json;

    const OWN_URL: &str = "http://localhost:8080";
    const OWN_URL_DOCKER: &str = "http://host.docker.internal:8080";

    // =========================================================================
    // Helpers
    // =========================================================================

    fn def(name: &str, param_type: ParameterType, default: Option<&str>) -> ParameterDefinition {
        ParameterDefinition {
            name: name.to_string(),
            title: name.to_string(),
            description: None,
            param_type,
            required: false,
            default_value: default.map(str::to_string),
        }
    }

    fn template_with_url(url: &str) -> ConnectorTemplateDto {
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

    fn template_with_body(body: &str) -> ConnectorTemplateDto {
        ConnectorTemplateDto {
            interaction: InteractionConfig::Pull(PullLifecycle {
                data_access: ProtocolSpec::Http(HttpSpec {
                    url_template: "https://api.example.com/resource".to_string(),
                    method: TemplateVecString::Value(vec!["POST".to_string()]),
                    headers: None,
                    body_template: Some(body.to_string()),
                }),
            }),
            ..template_with_url("https://api.example.com")
        }
    }

    fn build_with_defaults(defs: &[ParameterDefinition]) -> HashMap<String, serde_json::Value> {
        InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_default_parameters(defs)
            .unwrap()
            .build()
            .inner
    }

    fn build_with_sys(template: ConnectorTemplateDto) -> HashMap<String, serde_json::Value> {
        InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_system_parameters(&mut { template })
            .unwrap()
            .build()
            .inner
    }

    // =========================================================================
    // with_default_parameters
    // =========================================================================

    #[test]
    fn defaults_inserts_string_default() {
        let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
        assert_eq!(build_with_defaults(&defs)["REGION"], json!("us-east-1"));
    }

    #[test]
    fn defaults_inserts_int_default() {
        let defs = vec![def("TIMEOUT", ParameterType::Int, Some("30"))];
        assert_eq!(build_with_defaults(&defs)["TIMEOUT"], json!(30i64));
    }

    #[test]
    fn defaults_inserts_boolean_default() {
        let defs = vec![def("ENABLED", ParameterType::Boolean, Some("true"))];
        assert_eq!(build_with_defaults(&defs)["ENABLED"], json!(true));
    }

    #[test]
    fn defaults_inserts_vec_string_default() {
        let defs = vec![def(
            "TAGS",
            ParameterType::VecString,
            Some(r#"["prod","eu"]"#),
        )];
        assert_eq!(build_with_defaults(&defs)["TAGS"], json!(["prod", "eu"]));
    }

    #[test]
    fn defaults_inserts_map_string_default() {
        let defs = vec![def(
            "ENV",
            ParameterType::MapStringString,
            Some(r#"{"KEY":"val"}"#),
        )];
        assert_eq!(build_with_defaults(&defs)["ENV"], json!({"KEY": "val"}));
    }

    #[test]
    fn defaults_does_not_overwrite_existing_value() {
        let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
        let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_instance_parameters(&HashMap::from([("REGION".to_string(), json!("eu-west-1"))]))
            .with_default_parameters(&defs)
            .unwrap()
            .build()
            .inner;
        assert_eq!(
            inner["REGION"],
            json!("eu-west-1"),
            "instance value must win over default"
        );
    }

    #[test]
    fn defaults_skips_param_without_default() {
        let defs = vec![def("HOST", ParameterType::String, None)];
        assert!(
            !build_with_defaults(&defs).contains_key("HOST"),
            "no default - no injection"
        );
    }

    #[test]
    fn defaults_multiple_at_once() {
        let defs = vec![
            def("REGION", ParameterType::String, Some("us-east-1")),
            def("TIMEOUT", ParameterType::Int, Some("60")),
        ];
        let inner = build_with_defaults(&defs);
        assert_eq!(inner["REGION"], json!("us-east-1"));
        assert_eq!(inner["TIMEOUT"], json!(60i64));
    }

    #[test]
    fn defaults_err_on_malformed_int_default() {
        let defs = vec![def("PORT", ParameterType::Int, Some("not_a_number"))];
        assert!(InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_default_parameters(&defs)
            .is_err());
    }

    #[test]
    fn defaults_err_on_malformed_bool_default() {
        let defs = vec![def("FLAG", ParameterType::Boolean, Some("yes"))];
        assert!(InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_default_parameters(&defs)
            .is_err());
    }

    #[test]
    fn defaults_err_on_vec_default_that_is_not_array() {
        let defs = vec![def(
            "TAGS",
            ParameterType::VecString,
            Some(r#""not-an-array""#),
        )];
        assert!(InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_default_parameters(&defs)
            .is_err());
    }

    #[test]
    fn defaults_err_on_map_default_that_is_not_object() {
        let defs = vec![def(
            "ENV",
            ParameterType::MapStringString,
            Some(r#"["a","b"]"#),
        )];
        assert!(InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_default_parameters(&defs)
            .is_err());
    }

    // =========================================================================
    // with_system_parameters
    // =========================================================================

    #[test]
    fn sys_injects_urn_as_urn_string() {
        let inner = build_with_sys(template_with_url("https://api.example.com/{{__SYS_URN__}}"));
        let s = inner["SYS_URN"].as_str().expect("SYS_URN must be a string");
        assert!(s.starts_with("urn:uuid:"), "expected URN prefix, got: {s}");
    }

    #[test]
    fn sys_injects_token_as_uuid_string() {
        let inner = build_with_sys(template_with_body("{{__SYS_TOKEN__}}"));
        let s = inner["SYS_TOKEN"]
            .as_str()
            .expect("SYS_TOKEN must be a string");
        assert_eq!(s.len(), 36, "expected UUID string (36 chars), got: {s}");
    }

    #[test]
    fn sys_injects_timestamp_as_integer() {
        let inner = build_with_sys(template_with_body("ts={{__SYS_TIMESTAMP__}}"));
        let ts = inner["SYS_TIMESTAMP"]
            .as_i64()
            .expect("SYS_TIMESTAMP must be i64");
        assert!(ts > 1_580_000_000, "timestamp looks wrong: {ts}");
    }

    #[test]
    fn sys_injects_iso8601_as_rfc3339_string() {
        let inner = build_with_sys(template_with_url(
            "https://api.example.com/{{__SYS_ISO8601__}}",
        ));
        let s = inner["SYS_ISO8601"]
            .as_str()
            .expect("SYS_ISO8601 must be a string");
        assert!(s.contains('T'), "expected ISO8601 string, got: {s}");
    }

    #[test]
    fn sys_injects_own_url_from_config() {
        let inner = build_with_sys(template_with_url("{{__SYS_OWN_URL__}}/webhook"));
        assert_eq!(inner["SYS_OWN_URL"].as_str().unwrap(), OWN_URL);
    }

    #[test]
    fn sys_injects_own_url_docker_replaces_localhost() {
        let inner = build_with_sys(template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook"));
        assert_eq!(
            inner["SYS_OWN_URL_DOCKER"].as_str().unwrap(),
            OWN_URL_DOCKER
        );
    }

    #[test]
    fn sys_injects_own_url_docker_replaces_127_0_0_1() {
        let inner = InstanceParametersMapBuilder::new("http://127.0.0.1:8080".to_string())
            .with_system_parameters(&mut template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook"))
            .unwrap()
            .build()
            .inner;
        let resolved = inner["SYS_OWN_URL_DOCKER"].as_str().unwrap();
        assert!(
            !resolved.contains("127.0.0.1"),
            "127.0.0.1 must be replaced, got: {resolved}"
        );
        assert!(
            resolved.contains("host.docker.internal"),
            "expected host.docker.internal, got: {resolved}"
        );
    }

    #[test]
    fn sys_own_url_docker_with_remote_url_unchanged() {
        let remote = "https://my-connector.example.com:8080";
        let inner = InstanceParametersMapBuilder::new(remote.to_string())
            .with_system_parameters(&mut template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook"))
            .unwrap()
            .build()
            .inner;
        assert_eq!(inner["SYS_OWN_URL_DOCKER"].as_str().unwrap(), remote);
    }

    #[test]
    fn sys_does_not_overwrite_existing_value() {
        let pre_existing = "urn:uuid:pinned".to_string();
        let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_instance_parameters(&HashMap::from([(
                "SYS_URN".to_string(),
                json!(pre_existing.clone()),
            )]))
            .with_system_parameters(&mut template_with_url(
                "https://example.com/{{__SYS_URN__}}",
            ))
            .unwrap()
            .build()
            .inner;
        assert_eq!(inner["SYS_URN"].as_str().unwrap(), pre_existing);
    }

    #[test]
    fn sys_does_nothing_when_no_placeholders() {
        assert!(build_with_sys(template_with_url("https://api.example.com/data")).is_empty());
    }

    #[test]
    fn sys_injects_multiple_at_once() {
        let inner = build_with_sys(template_with_url(
            "{{__SYS_OWN_URL__}}/{{__SYS_URN__}}?ts={{__SYS_TIMESTAMP__}}",
        ));
        assert!(inner.contains_key("SYS_OWN_URL"));
        assert!(inner.contains_key("SYS_URN"));
        assert!(inner.contains_key("SYS_TIMESTAMP"));
    }

    // =========================================================================
    // Pipeline — instance - default - sys priority
    // =========================================================================

    #[test]
    fn pipeline_instance_wins_over_default() {
        let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
        let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_instance_parameters(&HashMap::from([("REGION".to_string(), json!("eu-west-1"))]))
            .with_default_parameters(&defs)
            .unwrap()
            .build()
            .inner;
        assert_eq!(inner["REGION"], json!("eu-west-1"));
    }

    #[test]
    fn pipeline_default_fills_gap_left_by_instance() {
        let defs = vec![
            def("HOST", ParameterType::String, Some("localhost")),
            def("PORT", ParameterType::Int, Some("8080")),
        ];
        let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_instance_parameters(&HashMap::from([("HOST".to_string(), json!("my-host"))]))
            .with_default_parameters(&defs)
            .unwrap()
            .build()
            .inner;
        assert_eq!(inner["HOST"], json!("my-host"), "instance wins");
        assert_eq!(inner["PORT"], json!(8080i64), "default fills the gap");
    }

    #[test]
    fn pipeline_sys_does_not_overwrite_instance() {
        let pre_existing = "urn:uuid:pinned".to_string();
        let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
            .with_instance_parameters(&HashMap::from([(
                "SYS_URN".to_string(),
                json!(pre_existing.clone()),
            )]))
            .with_system_parameters(&mut template_with_url(
                "https://example.com/{{__SYS_URN__}}",
            ))
            .unwrap()
            .build()
            .inner;
        assert_eq!(inner["SYS_URN"].as_str().unwrap(), pre_existing);
    }
}
