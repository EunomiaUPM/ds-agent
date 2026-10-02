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

/// Fills an instance's runtime placeholders from the request, the keystore and the ingress URL.
pub struct RuntimeParametersResolver<'a> {
    connector_instance: &'a ConnectorInstanceDto,
    runtime_params: &'a serde_json::Value,
    ingress_url: Option<String>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
    /// Tenant whose keystore resolves the placeholders; set together with `keystore`.
    keystore_tenant: String,
}

impl<'a> RuntimeParametersResolver<'a> {
    /// `runtime_params` holds the values sent with the transfer.
    pub fn new(
        connector_instance: &'a ConnectorInstanceDto,
        runtime_params: &'a serde_json::Value,
    ) -> Self {
        Self {
            connector_instance,
            runtime_params,
            ingress_url: None,
            keystore: None,
            keystore_tenant: String::new(),
        }
    }

    /// Value for `{{__RUNTIME_INGRESS__}}`; empty when unset.
    pub fn with_ingress(mut self, url: Option<impl Into<String>>) -> Self {
        self.ingress_url = url.map(Into::into);
        self
    }

    /// Resolves keystore placeholders against `tenant_id`'s parameters and secrets.
    pub fn with_keystore(mut self, lookup: Arc<dyn KeystoreLookup>, tenant_id: &str) -> Self {
        self.keystore = Some(lookup);
        self.keystore_tenant = tenant_id.to_string();
        self
    }

    /// Copy of the instance with every placeholder it can resolve filled in.
    pub async fn resolve(&self) -> Outcome<ConnectorInstanceDto> {
        // Serialize once; reuse for keystore regex scan and for in-place mutation.
        let mut value = serde_json::to_value(self.connector_instance).map_err(|e| {
            Errors::crazy(format!("Failed to serialize connector instance: {e}"), None)
        })?;
        let json = value.to_string();

        let (param_cache, secret_cache) = tokio::join!(
            self.fetch_keystore(template_runtime_parameter_regex(), &json, true),
            self.fetch_keystore(template_runtime_secret_regex(), &json, false)
        );

        self.resolve_value(&mut value, &param_cache, &secret_cache);
        serde_json::from_value(value).map_err(|e| {
            Errors::crazy(
                format!("Failed to deserialize resolved connector instance: {e}"),
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
                let tenant = self.keystore_tenant.clone();
                async move {
                    let val = if is_param {
                        ks.get_parameter(&tenant, &key).await
                    } else {
                        ks.get_secret(&tenant, &key).await
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
