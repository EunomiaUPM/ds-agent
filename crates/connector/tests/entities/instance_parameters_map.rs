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

//! InstanceParametersMapBuilder: typed defaults, system parameters and their precedence.

use connector::entities::connector_template::ConnectorMetadata;
use connector::entities::connector_template::ConnectorTemplateDto;
use connector::entities::parameters::{ParameterDefinition, ParameterType};
use serde_json::json;
use std::collections::HashMap;

use connector::entities::interaction::{InteractionConfig, PullLifecycle};
use connector::entities::parameters::instance_parameters_map::*;
use connector::entities::parameters::TemplateVecString;
use connector::entities::resource::HttpSpec;
use connector::{AuthenticationConfig, ProtocolSpec};

const OWN_URL: &str = "http://localhost:8080";
const OWN_URL_DOCKER: &str = "http://host.docker.internal:8080";

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
        .collect()
}

fn build_with_sys(template: ConnectorTemplateDto) -> HashMap<String, serde_json::Value> {
    InstanceParametersMapBuilder::new(OWN_URL.to_string())
        .with_system_parameters(&mut { template })
        .unwrap()
        .build()
        .collect()
}

/// A String default is inserted as a JSON string.
#[test]
fn defaults_inserts_string_default() {
    let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
    assert_eq!(build_with_defaults(&defs)["REGION"], json!("us-east-1"));
}

/// An Int default is parsed into a JSON integer.
#[test]
fn defaults_inserts_int_default() {
    let defs = vec![def("TIMEOUT", ParameterType::Int, Some("30"))];
    assert_eq!(build_with_defaults(&defs)["TIMEOUT"], json!(30i64));
}

/// A Boolean default is parsed into a JSON bool.
#[test]
fn defaults_inserts_boolean_default() {
    let defs = vec![def("ENABLED", ParameterType::Boolean, Some("true"))];
    assert_eq!(build_with_defaults(&defs)["ENABLED"], json!(true));
}

/// A VecString default is parsed into a JSON array.
#[test]
fn defaults_inserts_vec_string_default() {
    let defs = vec![def(
        "TAGS",
        ParameterType::VecString,
        Some(r#"["prod","eu"]"#),
    )];
    assert_eq!(build_with_defaults(&defs)["TAGS"], json!(["prod", "eu"]));
}

/// A MapStringString default is parsed into a JSON object.
#[test]
fn defaults_inserts_map_string_default() {
    let defs = vec![def(
        "ENV",
        ParameterType::MapStringString,
        Some(r#"{"KEY":"val"}"#),
    )];
    assert_eq!(build_with_defaults(&defs)["ENV"], json!({"KEY": "val"}));
}

/// A default never replaces a value the instance already set.
#[test]
fn defaults_does_not_overwrite_existing_value() {
    let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
    let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
        .with_instance_parameters(&HashMap::from([("REGION".to_string(), json!("eu-west-1"))]))
        .with_default_parameters(&defs)
        .unwrap()
        .build()
        .collect();
    assert_eq!(
        inner["REGION"],
        json!("eu-west-1"),
        "instance value must win over default"
    );
}

/// A parameter without default is left out.
#[test]
fn defaults_skips_param_without_default() {
    let defs = vec![def("HOST", ParameterType::String, None)];
    assert!(
        !build_with_defaults(&defs).contains_key("HOST"),
        "no default - no injection"
    );
}

/// Several defaults are inserted in one pass.
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

/// An Int default that is not a number is an error.
#[test]
fn defaults_err_on_malformed_int_default() {
    let defs = vec![def("PORT", ParameterType::Int, Some("not_a_number"))];
    assert!(InstanceParametersMapBuilder::new(OWN_URL.to_string())
        .with_default_parameters(&defs)
        .is_err());
}

/// A Boolean default other than true or false is an error.
#[test]
fn defaults_err_on_malformed_bool_default() {
    let defs = vec![def("FLAG", ParameterType::Boolean, Some("yes"))];
    assert!(InstanceParametersMapBuilder::new(OWN_URL.to_string())
        .with_default_parameters(&defs)
        .is_err());
}

/// A VecString default that is not a JSON array is an error.
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

/// A MapStringString default that is not a JSON object is an error.
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

/// `__SYS_URN__` becomes a fresh `urn:uuid:` string.
#[test]
fn sys_injects_urn_as_urn_string() {
    let inner = build_with_sys(template_with_url("https://api.example.com/{{__SYS_URN__}}"));
    let s = inner["SYS_URN"].as_str().expect("SYS_URN must be a string");
    assert!(s.starts_with("urn:uuid:"), "expected URN prefix, got: {s}");
}

/// `__SYS_TOKEN__` becomes a fresh UUID string.
#[test]
fn sys_injects_token_as_uuid_string() {
    let inner = build_with_sys(template_with_body("{{__SYS_TOKEN__}}"));
    let s = inner["SYS_TOKEN"]
        .as_str()
        .expect("SYS_TOKEN must be a string");
    assert_eq!(s.len(), 36, "expected UUID string (36 chars), got: {s}");
}

/// `__SYS_TIMESTAMP__` becomes the current Unix time as an integer.
#[test]
fn sys_injects_timestamp_as_integer() {
    let inner = build_with_sys(template_with_body("ts={{__SYS_TIMESTAMP__}}"));
    let ts = inner["SYS_TIMESTAMP"]
        .as_i64()
        .expect("SYS_TIMESTAMP must be i64");
    assert!(ts > 1_580_000_000, "timestamp looks wrong: {ts}");
}

/// `__SYS_ISO8601__` becomes the current time as an RFC 3339 string.
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

/// `__SYS_OWN_URL__` becomes the agent's own URL.
#[test]
fn sys_injects_own_url_from_config() {
    let inner = build_with_sys(template_with_url("{{__SYS_OWN_URL__}}/webhook"));
    assert_eq!(inner["SYS_OWN_URL"].as_str().unwrap(), OWN_URL);
}

/// `__SYS_OWN_URL_DOCKER__` swaps `localhost` for `host.docker.internal`.
#[test]
fn sys_injects_own_url_docker_replaces_localhost() {
    let inner = build_with_sys(template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook"));
    assert_eq!(
        inner["SYS_OWN_URL_DOCKER"].as_str().unwrap(),
        OWN_URL_DOCKER
    );
}

/// `__SYS_OWN_URL_DOCKER__` swaps `127.0.0.1` for `host.docker.internal`.
#[test]
fn sys_injects_own_url_docker_replaces_127_0_0_1() {
    let inner = InstanceParametersMapBuilder::new("http://127.0.0.1:8080".to_string())
        .with_system_parameters(&mut template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook"))
        .unwrap()
        .build()
        .collect();
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

/// `__SYS_OWN_URL_DOCKER__` keeps a non-local URL as is.
#[test]
fn sys_own_url_docker_with_remote_url_unchanged() {
    let remote = "https://my-connector.example.com:8080";
    let inner = InstanceParametersMapBuilder::new(remote.to_string())
        .with_system_parameters(&mut template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook"))
        .unwrap()
        .build()
        .collect();
    assert_eq!(inner["SYS_OWN_URL_DOCKER"].as_str().unwrap(), remote);
}

/// A system parameter never replaces a value the instance already set.
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
        .collect();
    assert_eq!(inner["SYS_URN"].as_str().unwrap(), pre_existing);
}

/// A template without `__SYS_*__` placeholders adds nothing.
#[test]
fn sys_does_nothing_when_no_placeholders() {
    assert!(build_with_sys(template_with_url("https://api.example.com/data")).is_empty());
}

/// Every system placeholder in a template is filled in one pass.
#[test]
fn sys_injects_multiple_at_once() {
    let inner = build_with_sys(template_with_url(
        "{{__SYS_OWN_URL__}}/{{__SYS_URN__}}?ts={{__SYS_TIMESTAMP__}}",
    ));
    assert!(inner.contains_key("SYS_OWN_URL"));
    assert!(inner.contains_key("SYS_URN"));
    assert!(inner.contains_key("SYS_TIMESTAMP"));
}

/// Across the whole pipeline, an instance value wins over a default.
#[test]
fn pipeline_instance_wins_over_default() {
    let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
    let inner = InstanceParametersMapBuilder::new(OWN_URL.to_string())
        .with_instance_parameters(&HashMap::from([("REGION".to_string(), json!("eu-west-1"))]))
        .with_default_parameters(&defs)
        .unwrap()
        .build()
        .collect();
    assert_eq!(inner["REGION"], json!("eu-west-1"));
}

/// Defaults fill only the parameters the instance left unset.
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
        .collect();
    assert_eq!(inner["HOST"], json!("my-host"), "instance wins");
    assert_eq!(inner["PORT"], json!(8080i64), "default fills the gap");
}

/// Across the whole pipeline, a system parameter never replaces an instance value.
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
        .collect();
    assert_eq!(inner["SYS_URN"].as_str().unwrap(), pre_existing);
}
