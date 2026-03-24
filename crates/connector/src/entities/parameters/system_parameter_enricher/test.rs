use super::*;
use crate::entities::connector_template::{ConnectorMetadata, ConnectorTemplateDto};
use crate::entities::interaction::{InteractionConfig, PullLifecycle};
use crate::entities::resource::HttpSpec;
use crate::{AuthenticationConfig, ProtocolSpec, TemplateVecString};
use serde_json::json;
use std::collections::HashMap;

const OWN_URL: &str = "http://localhost:8080";
const OWN_URL_DOCKER: &str = "http://host.docker.internal:8080";

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

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

fn enrich(template: &ConnectorTemplateDto) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    SysParameterEnricher::new(template, OWN_URL)
        .enrich(&mut params)
        .unwrap();
    params
}

// -------------------------------------------------------------------------
// SYS_URN
// -------------------------------------------------------------------------

#[test]
fn injects_sys_urn_as_urn_string() {
    let template = template_with_url("https://api.example.com/{{__SYS_URN__}}");
    let params = enrich(&template);

    let s = params["SYS_URN"]
        .as_str()
        .expect("SYS_URN must be a string");
    assert!(
        s.starts_with("urn:uuid:"),
        "expected URN prefix, got: {}",
        s
    );
}

// -------------------------------------------------------------------------
// SYS_TOKEN
// -------------------------------------------------------------------------

#[test]
fn injects_sys_token_as_uuid_string() {
    let template = template_with_body("{{__SYS_TOKEN__}}");
    let params = enrich(&template);

    let s = params["SYS_TOKEN"]
        .as_str()
        .expect("SYS_TOKEN must be a string");
    assert_eq!(s.len(), 36, "expected UUID string, got: {}", s);
}

// -------------------------------------------------------------------------
// SYS_TIMESTAMP
// -------------------------------------------------------------------------

#[test]
fn injects_sys_timestamp_as_integer() {
    let template = template_with_body("since={{__SYS_TIMESTAMP__}}");
    let params = enrich(&template);

    let ts = params["SYS_TIMESTAMP"]
        .as_i64()
        .expect("SYS_TIMESTAMP must be an integer");
    assert!(ts > 1_580_000_000, "timestamp looks wrong: {}", ts);
}

// -------------------------------------------------------------------------
// SYS_ISO8601
// -------------------------------------------------------------------------

#[test]
fn injects_sys_iso8601_as_rfc3339_string() {
    let template = template_with_url("https://api.example.com/{{__SYS_ISO8601__}}");
    let params = enrich(&template);

    let s = params["SYS_ISO8601"]
        .as_str()
        .expect("SYS_ISO8601 must be a string");
    assert!(s.contains('T'), "expected ISO8601 string, got: {}", s);
}

// -------------------------------------------------------------------------
// SYS_OWN_URL
// -------------------------------------------------------------------------

#[test]
fn injects_sys_own_url_from_config() {
    let template = template_with_url("{{__SYS_OWN_URL__}}/webhook");
    let params = enrich(&template);

    assert_eq!(
        params["SYS_OWN_URL"].as_str().unwrap(),
        OWN_URL,
        "SYS_OWN_URL must match the configured own_url"
    );
}

#[test]
fn injects_sys_own_url_docker_replaces_localhost() {
    let template = template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook");
    let params = enrich(&template);

    assert_eq!(
        params["SYS_OWN_URL_DOCKER"].as_str().unwrap(),
        OWN_URL_DOCKER,
        "SYS_OWN_URL_DOCKER must have localhost replaced"
    );
}

#[test]
fn injects_sys_own_url_docker_replaces_127_0_0_1() {
    let template = template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook");
    let url_with_ip = "http://127.0.0.1:8080";
    let mut params = HashMap::new();
    SysParameterEnricher::new(&template, url_with_ip)
        .enrich(&mut params)
        .unwrap();

    let resolved = params["SYS_OWN_URL_DOCKER"].as_str().unwrap();
    assert!(
        !resolved.contains("127.0.0.1"),
        "127.0.0.1 must be replaced, got: {}",
        resolved
    );
    assert!(
        resolved.contains("host.docker.internal"),
        "expected host.docker.internal, got: {}",
        resolved
    );
}

#[test]
fn sys_own_url_with_non_localhost_url_unchanged_for_docker() {
    let template = template_with_url("{{__SYS_OWN_URL_DOCKER__}}/webhook");
    let remote_url = "https://my-connector.example.com:8080";
    let mut params = HashMap::new();
    SysParameterEnricher::new(&template, remote_url)
        .enrich(&mut params)
        .unwrap();

    // Non-localhost URLs should pass through unchanged
    assert_eq!(
        params["SYS_OWN_URL_DOCKER"].as_str().unwrap(),
        remote_url,
        "non-localhost URLs must pass through unchanged for docker variant"
    );
}

// -------------------------------------------------------------------------
// Idempotency — must not overwrite existing values
// -------------------------------------------------------------------------

#[test]
fn does_not_overwrite_existing_sys_urn() {
    let template = template_with_url("https://api.example.com/{{__SYS_URN__}}");
    let pre_existing = "urn:uuid:pre-existing-value".to_string();
    let mut params = HashMap::from([("SYS_URN".to_string(), json!(pre_existing.clone()))]);

    SysParameterEnricher::new(&template, OWN_URL)
        .enrich(&mut params)
        .unwrap();

    assert_eq!(params["SYS_URN"].as_str().unwrap(), pre_existing);
}

#[test]
fn does_not_overwrite_existing_sys_own_url() {
    let template = template_with_url("{{__SYS_OWN_URL__}}/webhook");
    let pre_existing = "https://override.example.com".to_string();
    let mut params = HashMap::from([("SYS_OWN_URL".to_string(), json!(pre_existing.clone()))]);

    SysParameterEnricher::new(&template, OWN_URL)
        .enrich(&mut params)
        .unwrap();

    assert_eq!(params["SYS_OWN_URL"].as_str().unwrap(), pre_existing);
}

// -------------------------------------------------------------------------
// No placeholders → no injection
// -------------------------------------------------------------------------

#[test]
fn does_nothing_when_template_has_no_sys_placeholders() {
    let template = template_with_url("https://api.example.com/data");
    let params = enrich(&template);

    assert!(params.is_empty());
}

// -------------------------------------------------------------------------
// Multiple SYS placeholders in one template
// -------------------------------------------------------------------------

#[test]
fn injects_multiple_sys_params_from_same_template() {
    let template = ConnectorTemplateDto {
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "{{__SYS_OWN_URL__}}/{{__SYS_URN__}}?ts={{__SYS_TIMESTAMP__}}"
                    .to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        ..template_with_url("https://api.example.com")
    };
    let params = enrich(&template);

    assert!(params.contains_key("SYS_OWN_URL"));
    assert!(params.contains_key("SYS_URN"));
    assert!(params.contains_key("SYS_TIMESTAMP"));
}