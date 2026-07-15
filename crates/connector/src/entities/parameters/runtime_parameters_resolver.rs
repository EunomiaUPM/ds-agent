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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::entities::parameters::jq::run_jq;
use crate::entities::parameters::keystore_lookup::KeystoreLookup;
use crate::entities::parameters::{
    template_runtime_json_regex, template_runtime_parameter_regex, template_runtime_secret_regex,
};
use crate::ConnectorInstanceDto;
use regex::Regex;
use ymir::errors::{Errors, Outcome};

pub struct RuntimeParametersResolver<'a> {
    connector_instance: &'a ConnectorInstanceDto,
    runtime_params: &'a serde_json::Value,
    ingress_url: Option<String>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
}

impl<'a> RuntimeParametersResolver<'a> {
    pub fn new(
        connector_instance: &'a ConnectorInstanceDto,
        runtime_params: &'a serde_json::Value,
    ) -> Self {
        Self {
            connector_instance,
            runtime_params,
            ingress_url: None,
            keystore: None,
        }
    }

    pub fn with_ingress(mut self, url: Option<impl Into<String>>) -> Self {
        self.ingress_url = url.map(Into::into);
        self
    }

    pub fn with_keystore(mut self, lookup: Arc<dyn KeystoreLookup>) -> Self {
        self.keystore = Some(lookup);
        self
    }

    pub async fn resolve(&self) -> Outcome<ConnectorInstanceDto> {
        // Serialize once; reuse for keystore regex scan and for in-place mutation.
        let mut value = serde_json::to_value(self.connector_instance).map_err(|e| {
            Errors::crazy(
                format!("Failed to serialize connector instance: {}", e),
                None,
            )
        })?;
        let json = value.to_string();

        let (param_cache, secret_cache) = tokio::join!(
            self.fetch_keystore(template_runtime_parameter_regex(), &json, true),
            self.fetch_keystore(template_runtime_secret_regex(), &json, false)
        );

        self.resolve_value(&mut value, &param_cache, &secret_cache);
        serde_json::from_value(value).map_err(|e| {
            Errors::crazy(
                format!("Failed to deserialize resolved connector instance: {}", e),
                None,
            )
        })
    }

    async fn fetch_keystore(
        &self,
        re: &Regex,
        json: &str,
        is_param: bool,
    ) -> HashMap<String, serde_json::Value> {
        let Some(ks) = &self.keystore else {
            return HashMap::new();
        };
        let keys: HashSet<String> = re
            .captures_iter(json)
            .map(|cap| cap[1].to_string())
            .collect();
        let fetches: Vec<_> = keys
            .into_iter()
            .map(|key| {
                let ks = ks.clone();
                async move {
                    let val = if is_param {
                        ks.get_parameter(&key).await
                    } else {
                        ks.get_secret(&key).await
                    };
                    (key, val)
                }
            })
            .collect();
        futures_util::future::join_all(fetches)
            .await
            .into_iter()
            .filter_map(|(k, v)| v.map(|val| (k, val)))
            .collect()
    }

    fn resolve_value(
        &self,
        value: &mut serde_json::Value,
        params: &HashMap<String, serde_json::Value>,
        secrets: &HashMap<String, serde_json::Value>,
    ) {
        match value {
            serde_json::Value::String(s) => {
                if let Some(resolved) = self.resolve_string(s, params, secrets) {
                    *value = resolved;
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    self.resolve_value(v, params, secrets);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    self.resolve_value(v, params, secrets);
                }
            }
            _ => {}
        }
    }

    fn resolve_string(
        &self,
        raw: &str,
        params: &HashMap<String, serde_json::Value>,
        secrets: &HashMap<String, serde_json::Value>,
    ) -> Option<serde_json::Value> {
        const INGRESS_PLACEHOLDER: &str = "{{__RUNTIME_INGRESS__}}";

        let mut work = raw.to_string();
        let mut changed = false;

        // Step 1: INGRESS substitution.
        if work.contains(INGRESS_PLACEHOLDER) {
            work = work.replace(
                INGRESS_PLACEHOLDER,
                self.ingress_url.as_deref().unwrap_or(""),
            );
            changed = true;
        }

        // Step 2: PARAMETER — exact match preserves JSON type.
        if let Some(val) =
            Self::try_keystore_exact(&work, template_runtime_parameter_regex(), params)
        {
            return Some(val);
        }
        changed |= Self::apply_keystore_interpolation(
            &mut work,
            template_runtime_parameter_regex(),
            params,
        );

        // Step 3: SECRET — same pattern.
        if let Some(val) = Self::try_keystore_exact(&work, template_runtime_secret_regex(), secrets)
        {
            return Some(val);
        }
        changed |=
            Self::apply_keystore_interpolation(&mut work, template_runtime_secret_regex(), secrets);

        // Step 4: RUNTIME_JSON (jq evaluation).
        let re = template_runtime_json_regex();
        if re.is_match(&work) {
            if let Some(caps) = re.captures(&work) {
                // Exact match — preserve JSON type.
                if caps.get(0).map(|m| m.as_str()) == Some(work.as_str()) {
                    let expr = &caps[1];
                    return run_jq(expr, self.runtime_params.clone()).filter(|v| !v.is_null());
                }
            }
            // Interpolation — stringify each match.
            let snapshot = work.clone();
            let mut result = snapshot.clone();
            let mut json_changed = false;
            for caps in re.captures_iter(&snapshot) {
                let full_match = &caps[0];
                let expr = &caps[1];
                if let Some(val) = run_jq(expr, self.runtime_params.clone()) {
                    let s = match &val {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    result = result.replace(full_match, &s);
                    json_changed = true;
                }
            }
            if json_changed {
                return Some(serde_json::Value::String(result));
            }
            // No jq expression resolved — fall through to the `changed` check below.
        }

        if changed {
            Some(serde_json::Value::String(work))
        } else {
            None
        }
    }

    /// Returns the cached value when `current` is an exact (whole-string) match of one placeholder.
    fn try_keystore_exact(
        current: &str,
        re: &Regex,
        cache: &HashMap<String, serde_json::Value>,
    ) -> Option<serde_json::Value> {
        let caps = re.captures(current)?;
        if caps.get(0)?.as_str() != current {
            return None;
        }
        cache.get(&caps[1]).cloned()
    }

    /// Replaces all matching placeholders in `current` with their stringified cached values.
    /// Returns `true` if at least one substitution was made.
    fn apply_keystore_interpolation(
        current: &mut String,
        re: &Regex,
        cache: &HashMap<String, serde_json::Value>,
    ) -> bool {
        if !re.is_match(current) {
            return false;
        }
        let snapshot = current.clone();
        let mut changed = false;
        for caps in re.captures_iter(&snapshot) {
            let full_match = &caps[0];
            let key = &caps[1];
            if let Some(val) = cache.get(key) {
                let s = match val {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                *current = current.replace(full_match, &s);
                changed = true;
            }
        }
        changed
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
            distribution_id: Urn::from_str("urn:uuid:00000000-0000-0000-0000-000000000002")
                .unwrap(),
        }
    }

    async fn resolve_helper(
        instance: &ConnectorInstanceDto,
        params: serde_json::Value,
    ) -> ConnectorInstanceDto {
        RuntimeParametersResolver::new(instance, &params)
            .resolve()
            .await
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

    #[tokio::test]
    async fn resolves_interpolated_jq_expression() {
        let instance = push_instance(
            "https://api.example.com/{{__RUNTIME_JSON_{.subscribe.id}__}}/hook",
            None,
        );
        let params = json!({ "subscribe": { "id": "abc-123" } });
        let result = resolve_helper(&instance, params).await;
        assert_eq!(
            subscribe_url(&result),
            "https://api.example.com/abc-123/hook"
        );
    }

    #[tokio::test]
    async fn exact_placeholder_resolves_to_string() {
        let instance = push_instance("{{__RUNTIME_JSON_{.subscribe.id}__}}", None);
        let params = json!({ "subscribe": { "id": "resolved-id" } });
        let result = resolve_helper(&instance, params).await;
        assert_eq!(subscribe_url(&result), "resolved-id");
    }

    #[tokio::test]
    async fn leaves_unmatched_string_unchanged() {
        let url = "https://api.example.com/static";
        let instance = push_instance(url, None);
        let result = resolve_helper(&instance, json!({})).await;
        assert_eq!(subscribe_url(&result), url);
    }

    #[tokio::test]
    async fn missing_jq_path_leaves_placeholder_unchanged() {
        let raw = "{{__RUNTIME_JSON_{.subscribe.missing}__}}";
        let instance = push_instance(raw, None);
        let params = json!({ "subscribe": { "id": "x" } });
        let result = resolve_helper(&instance, params).await;
        assert_eq!(subscribe_url(&result), raw);
    }

    #[tokio::test]
    async fn supports_complex_jq_expression() {
        let instance = push_instance("{{__RUNTIME_JSON_{.items[0].name}__}}", None);
        let params = json!({ "items": [{ "name": "first" }, { "name": "second" }] });
        let result = resolve_helper(&instance, params).await;
        assert_eq!(subscribe_url(&result), "first");
    }

    #[tokio::test]
    async fn resolves_parameter_placeholder_exact() {
        struct FakeLookup;
        #[async_trait::async_trait]
        impl KeystoreLookup for FakeLookup {
            async fn get_parameter(&self, key: &str) -> Option<serde_json::Value> {
                if key == "/my/param" {
                    Some(json!("param-value"))
                } else {
                    None
                }
            }
            async fn get_secret(&self, _: &str) -> Option<serde_json::Value> {
                None
            }
        }
        let instance = push_instance("{{__RUNTIME_PARAMETER_{/my/param}__}}", None);
        let result = RuntimeParametersResolver::new(&instance, &json!({}))
            .with_keystore(Arc::new(FakeLookup))
            .resolve()
            .await
            .unwrap();
        assert_eq!(subscribe_url(&result), "param-value");
    }

    #[tokio::test]
    async fn resolves_secret_placeholder_interpolated() {
        struct FakeLookup;
        #[async_trait::async_trait]
        impl KeystoreLookup for FakeLookup {
            async fn get_parameter(&self, _: &str) -> Option<serde_json::Value> {
                None
            }
            async fn get_secret(&self, key: &str) -> Option<serde_json::Value> {
                if key == "/my/secret" {
                    Some(json!("s3cr3t"))
                } else {
                    None
                }
            }
        }
        let instance = push_instance(
            "https://host/{{__RUNTIME_SECRET_{/my/secret}__}}/path",
            None,
        );
        let result = RuntimeParametersResolver::new(&instance, &json!({}))
            .with_keystore(Arc::new(FakeLookup))
            .resolve()
            .await
            .unwrap();
        assert_eq!(subscribe_url(&result), "https://host/s3cr3t/path");
    }
}
