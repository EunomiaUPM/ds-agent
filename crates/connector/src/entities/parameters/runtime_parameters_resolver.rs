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

use crate::entities::parameters::template_runtime_json_regex;
use crate::ConnectorInstanceDto;
use ymir::errors::{Errors, Outcome};

/// Evaluate a jq expression against a JSON value. Returns the first output, or `None` on
/// parse/compile/runtime error (warnings are emitted via `tracing`).
fn run_jq(expr: &str, value: serde_json::Value) -> Option<serde_json::Value> {
    use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};

    let (f, errs) = jaq_parse::parse(expr, jaq_parse::main());
    if !errs.is_empty() {
        tracing::warn!("runtime_jq: parse errors for {:?}: {:?}", expr, errs);
        return None;
    }
    let f = f?;

    let mut defs = ParseCtx::new(Vec::new());
    let filter = defs.compile(f);
    if !defs.errs.is_empty() {
        tracing::warn!(
            "runtime_jq: {} compile error(s) for {:?}",
            defs.errs.len(),
            expr
        );
        return None;
    }

    let inputs = RcIter::new(core::iter::empty());
    let result = filter
        .run((Ctx::new([], &inputs), Val::from(value)))
        .next()
        .and_then(|r| r.ok())
        .map(serde_json::Value::from);
    result
}

pub struct RuntimeParametersResolver<'a> {
    connector_instance: &'a ConnectorInstanceDto,
    runtime_params: &'a serde_json::Value,
}

impl<'a> RuntimeParametersResolver<'a> {
    pub fn new(
        connector_instance: &'a ConnectorInstanceDto,
        runtime_params: &'a serde_json::Value,
    ) -> Self {
        Self {
            connector_instance,
            runtime_params,
        }
    }

    pub fn resolve(&self) -> Outcome<ConnectorInstanceDto> {
        let mut value = serde_json::to_value(self.connector_instance).map_err(|e| {
            Errors::crazy(
                format!("Failed to serialize connector instance: {}", e),
                None,
            )
        })?;
        self.resolve_value(&mut value);
        serde_json::from_value(value).map_err(|e| {
            Errors::crazy(
                format!("Failed to deserialize resolved connector instance: {}", e),
                None,
            )
        })
    }

    fn resolve_value(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                if let Some(resolved) = self.resolve_string(s) {
                    *value = resolved;
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    self.resolve_value(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    self.resolve_value(v);
                }
            }
            _ => {}
        }
    }

    /// Returns `Some` only when the string contains at least one `{{__RUNTIME_JSON_{...}__}}`.
    /// - Exact match  → runs jq and returns the raw output (preserves JSON type).
    /// - Interpolated → runs jq for each placeholder and replaces with its string form.
    fn resolve_string(&self, raw: &str) -> Option<serde_json::Value> {
        let re = template_runtime_json_regex();
        let captures = re.captures(raw)?;

        // Exact match: the whole string is a single placeholder → preserve JSON type.
        // null is treated as "not found" so the placeholder is left unchanged.
        if captures.get(0)?.as_str() == raw {
            let expr = &captures[1];
            return run_jq(expr, self.runtime_params.clone()).filter(|v| !v.is_null());
        }

        // Interpolation: one or more placeholders embedded in text.
        let mut result = raw.to_string();
        for caps in re.captures_iter(raw) {
            let full_match = &caps[0];
            let expr = &caps[1];
            if let Some(val) = run_jq(expr, self.runtime_params.clone()) {
                let s = match &val {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                result = result.replace(full_match, &s);
            }
        }
        Some(serde_json::Value::String(result))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::auth_config::AuthenticationConfig;
    use crate::entities::interaction::{InteractionConfig, PushLifecycle};
    use crate::entities::resource::HttpSpec;
    use crate::{ConnectorMetadata, ProtocolSpec, TemplateVecString};
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

    fn resolve(instance: &ConnectorInstanceDto, params: serde_json::Value) -> ConnectorInstanceDto {
        RuntimeParametersResolver::new(instance, &params)
            .resolve()
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

    #[test]
    fn resolves_interpolated_jq_expression() {
        let instance = push_instance(
            "https://api.example.com/{{__RUNTIME_JSON_{.subscribe.id}__}}/hook",
            None,
        );
        let params = json!({ "subscribe": { "id": "abc-123" } });
        let result = resolve(&instance, params);
        assert_eq!(subscribe_url(&result), "https://api.example.com/abc-123/hook");
    }

    #[test]
    fn exact_placeholder_resolves_to_string() {
        let instance = push_instance("{{__RUNTIME_JSON_{.subscribe.id}__}}", None);
        let params = json!({ "subscribe": { "id": "resolved-id" } });
        let result = resolve(&instance, params);
        assert_eq!(subscribe_url(&result), "resolved-id");
    }

    #[test]
    fn leaves_unmatched_string_unchanged() {
        let url = "https://api.example.com/static";
        let instance = push_instance(url, None);
        let result = resolve(&instance, json!({}));
        assert_eq!(subscribe_url(&result), url);
    }

    #[test]
    fn missing_jq_path_leaves_placeholder_unchanged() {
        let raw = "{{__RUNTIME_JSON_{.subscribe.missing}__}}";
        let instance = push_instance(raw, None);
        let params = json!({ "subscribe": { "id": "x" } });
        let result = resolve(&instance, params);
        // null result → placeholder left as-is
        assert_eq!(subscribe_url(&result), raw);
    }

    #[test]
    fn supports_complex_jq_expression() {
        let instance = push_instance("{{__RUNTIME_JSON_{.items[0].name}__}}", None);
        let params = json!({ "items": [{ "name": "first" }, { "name": "second" }] });
        let result = resolve(&instance, params);
        assert_eq!(subscribe_url(&result), "first");
    }
}