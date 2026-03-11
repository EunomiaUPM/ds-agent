//! Injects `SYS_*` runtime values for every `{{__SYS_*__}}` placeholder that
//! is actually referenced inside a [`ConnectorTemplateDto`].
//!
//! # How it works
//!
//! The enricher performs two passes over the template:
//!
//! 1. **Discovery** — [`ParameterExtractorVisitor`] walks the whole DTO and
//!    feeds each templatable field to [`SystemParameterExtractor`], which
//!    collects every `{{__SYS_*__}}` match together with its
//!    [`SysParameterType`].
//!
//! 2. **Injection** — for each discovered placeholder, a runtime value is
//!    computed and inserted into the parameter map via [`HashMap::entry`] so
//!    that user-supplied overrides are never silently replaced.
//!
//! # `SysOwnUrl` resolution
//!
//! `{{__SYS_OWN_URL__}}` resolves to the service's own base URL as configured
//! (e.g. `https://my-connector.example.com:8080`).
//!
//! `{{__SYS_OWN_URL_DOCKER__}}` resolves to the same URL but with `localhost`
//! and `127.0.0.1` replaced by `host.docker.internal`, making the address
//! reachable from inside a Docker container.
//!
//! Both require the `own_url` string to be injected at construction time via
//! [`SysParameterEnricher::new`].
//!
//! [`ConnectorTemplateDto`]: crate::entities::connector_template::ConnectorTemplateDto

use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::parameters::SysParameterType;
use crate::entities::parameters::system_parameter_extractor::SystemParameterExtractor;
use crate::entities::parameters::template_parameters_visitor::ParameterExtractorVisitor;
use crate::entities::parameters::ParameterEnricher;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Enriches the parameter map with `SYS_*` runtime values.
///
/// Construct with a reference to the connector template and the service's own
/// base URL, then call [`ParameterEnricher::enrich`].
///
/// # Example
///
/// ```ignore
/// let mut params = instance_dto.parameters.clone();
/// SysParameterEnricher::new(&template_spec, &self.own_url).enrich(&mut params)?;
/// ```
pub struct SysParameterEnricher<'a> {
    template: &'a ConnectorTemplateDto,
    /// The service's own base URL (e.g. `https://my-connector.example.com:8080`).
    /// Used to resolve `{{__SYS_OWN_URL__}}` and `{{__SYS_OWN_URL_DOCKER__}}`.
    own_url: &'a str,
}

impl<'a> SysParameterEnricher<'a> {
    pub fn new(template: &'a ConnectorTemplateDto, own_url: &'a str) -> Self {
        Self { template, own_url }
    }

    /// Compute the runtime [`Value`] for a given [`SysParameterType`].
    fn resolve_sys_value(&self, content_type: &SysParameterType) -> Option<Value> {
        match content_type {
            SysParameterType::SysUrn => {
                let nss = uuid::Uuid::new_v4().to_string();
                let urn = urn::UrnBuilder::new("uuid", &nss).build().ok()?;
                Some(json!(urn.to_string()))
            }
            SysParameterType::SysToken => Some(json!(uuid::Uuid::new_v4().to_string())),
            SysParameterType::SysTimestamp => Some(json!(Utc::now().timestamp())),
            SysParameterType::SysIso8601 => Some(json!(Utc::now().to_rfc3339())),
            // The regular own URL — returned as-is from config.
            SysParameterType::SysOwnUrl { host_docker_internal: false } => {
                Some(json!(self.own_url))
            }
            // The Docker variant — replace localhost / 127.0.0.1 so that the
            // address is reachable from inside a container.
            SysParameterType::SysOwnUrl { host_docker_internal: true } => {
                let docker_url = self
                    .own_url
                    .replace("localhost", "host.docker.internal")
                    .replace("127.0.0.1", "host.docker.internal");
                Some(json!(docker_url))
            }
        }
    }
}

impl ParameterEnricher for SysParameterEnricher<'_> {
    /// Scans the template for `{{__SYS_*__}}` placeholders and inserts a
    /// runtime value for each one that does not already exist in `params`.
    fn enrich(&self, params: &mut HashMap<String, Value>) -> anyhow::Result<()> {
        let mut extractor = SystemParameterExtractor::new();
        // Clone the template so we can satisfy the &mut requirement of
        // ParameterExtractorVisitor without modifying the original.
        let mut tmpl = self.template.clone();
        ParameterExtractorVisitor::new(&mut extractor).extract(&mut tmpl);

        for found in extractor.found_sys_parameters() {
            if let Some(value) = self.resolve_sys_value(&found.content_type) {
                params.entry(found.name.clone()).or_insert(value);
            }
        }
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod test {
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
        SysParameterEnricher::new(template, OWN_URL).enrich(&mut params).unwrap();
        params
    }

    // -------------------------------------------------------------------------
    // SYS_URN
    // -------------------------------------------------------------------------

    #[test]
    fn injects_sys_urn_as_urn_string() {
        let template = template_with_url("https://api.example.com/{{__SYS_URN__}}");
        let params = enrich(&template);

        let s = params["SYS_URN"].as_str().expect("SYS_URN must be a string");
        assert!(s.starts_with("urn:uuid:"), "expected URN prefix, got: {}", s);
    }

    // -------------------------------------------------------------------------
    // SYS_TOKEN
    // -------------------------------------------------------------------------

    #[test]
    fn injects_sys_token_as_uuid_string() {
        let template = template_with_body("{{__SYS_TOKEN__}}");
        let params = enrich(&template);

        let s = params["SYS_TOKEN"].as_str().expect("SYS_TOKEN must be a string");
        assert_eq!(s.len(), 36, "expected UUID string, got: {}", s);
    }

    // -------------------------------------------------------------------------
    // SYS_TIMESTAMP
    // -------------------------------------------------------------------------

    #[test]
    fn injects_sys_timestamp_as_integer() {
        let template = template_with_body("since={{__SYS_TIMESTAMP__}}");
        let params = enrich(&template);

        let ts = params["SYS_TIMESTAMP"].as_i64().expect("SYS_TIMESTAMP must be an integer");
        assert!(ts > 1_580_000_000, "timestamp looks wrong: {}", ts);
    }

    // -------------------------------------------------------------------------
    // SYS_ISO8601
    // -------------------------------------------------------------------------

    #[test]
    fn injects_sys_iso8601_as_rfc3339_string() {
        let template = template_with_url("https://api.example.com/{{__SYS_ISO8601__}}");
        let params = enrich(&template);

        let s = params["SYS_ISO8601"].as_str().expect("SYS_ISO8601 must be a string");
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
        SysParameterEnricher::new(&template, url_with_ip).enrich(&mut params).unwrap();

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
        SysParameterEnricher::new(&template, remote_url).enrich(&mut params).unwrap();

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

        SysParameterEnricher::new(&template, OWN_URL).enrich(&mut params).unwrap();

        assert_eq!(params["SYS_URN"].as_str().unwrap(), pre_existing);
    }

    #[test]
    fn does_not_overwrite_existing_sys_own_url() {
        let template = template_with_url("{{__SYS_OWN_URL__}}/webhook");
        let pre_existing = "https://override.example.com".to_string();
        let mut params = HashMap::from([("SYS_OWN_URL".to_string(), json!(pre_existing.clone()))]);

        SysParameterEnricher::new(&template, OWN_URL).enrich(&mut params).unwrap();

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
}
