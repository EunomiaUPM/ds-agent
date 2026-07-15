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

use connector::{template_runtime_secret_regex, ApiKeyLocation, KeystoreLookup};
use keystore::{Key, KeyPrefix, SecretStore, SecretValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ymir::errors::Outcome;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DataplaneRuntime {
    pub auth: ResolvedAuthCredentials,
    pub subscription: serde_json::Value,
    pub unsubscription: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolvedAuthCredentials {
    #[default]
    NoAuth,
    BearerToken {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    OAuth2 {
        access_token: String,
        token_type: String,
        /// Unix timestamp (seconds) at which the token expires, if known.
        expires_at: Option<u64>,
        /// Refresh token returned by the token endpoint, if any.
        refresh_token: Option<String>,
    },
}

/// Handles transparent vaulting of sensitive fields in a [`DataplaneRuntime`].
///
/// - [`export`](RuntimeSecretVault::export): writes plaintext credentials to the secret store
///   and replaces them with `{{__RUNTIME_SECRET_{path}__}}` placeholders before DB persistence.
/// - [`resolve`](RuntimeSecretVault::resolve): reads the stored values back and substitutes
///   the placeholders with the real credentials before use.
/// - [`cleanup`](RuntimeSecretVault::cleanup): deletes all vault entries for a transfer on termination.
pub struct RuntimeSecretVault<'a> {
    store: &'a dyn SecretStore,
}

impl<'a> RuntimeSecretVault<'a> {
    pub fn new(store: &'a dyn SecretStore) -> Self {
        Self { store }
    }

    /// Vaults ephemeral credentials in `runtime` that are not owned by the connector config.
    ///
    /// Only `OAuth2` tokens are vaulted — access/refresh tokens are acquired at runtime and
    /// have no persistent home in the connector. Static connector credentials (`BearerToken`,
    /// `ApiKey`, `BasicAuth`) are passed through unchanged: their source of truth is the
    /// connector's own `SecretString`, so duplicating them under `/runtime/<id>/...` would
    /// create an unnecessary copy. Returns an error if any vault write fails.
    pub async fn export(
        &self,
        runtime: &DataplaneRuntime,
        transfer_id: &str,
    ) -> Outcome<DataplaneRuntime> {
        let prefix = Self::path_prefix(transfer_id);

        let auth = match &runtime.auth {
            ResolvedAuthCredentials::NoAuth => ResolvedAuthCredentials::NoAuth,

            // Static connector credentials: vault the literal value so the DB never stores
            // the secret in plain text. If the value is already a RUNTIME_SECRET placeholder
            // (i.e. the connector's SecretString points directly into the keystore), pass it
            // through — the proxy resolves it at request time without an intermediate vault entry.
            ResolvedAuthCredentials::BearerToken { token } => {
                ResolvedAuthCredentials::BearerToken {
                    token: token.clone(),
                }
            }

            ResolvedAuthCredentials::ApiKey {
                key,
                value,
                location,
            } => ResolvedAuthCredentials::ApiKey {
                key: key.clone(),
                value: value.clone(),
                location: location.clone(),
            },

            ResolvedAuthCredentials::BasicAuth { username, password } => {
                ResolvedAuthCredentials::BasicAuth {
                    username: username.clone(),
                    password: password.clone(),
                }
            }

            // OAuth2 tokens are ephemeral — not stored in the connector config — so they
            // must be vaulted to survive across continuation events.
            ResolvedAuthCredentials::OAuth2 {
                access_token,
                token_type,
                expires_at,
                refresh_token,
            } => {
                let at_path = format!("{}/access-token", prefix);

                let resolved_refresh = if let Some(rt) = refresh_token {
                    let rt_path = format!("{}/refresh-token", prefix);
                    let (r1, r2) = tokio::join!(
                        self.upsert(&at_path, json!(access_token)),
                        self.upsert(&rt_path, json!(rt))
                    );
                    r1?;
                    r2?;
                    Some(Self::placeholder(&rt_path))
                } else {
                    self.upsert(&at_path, json!(access_token)).await?;
                    None
                };

                ResolvedAuthCredentials::OAuth2 {
                    access_token: Self::placeholder(&at_path),
                    token_type: token_type.clone(),
                    expires_at: *expires_at,
                    refresh_token: resolved_refresh,
                }
            }
        };

        Ok(DataplaneRuntime {
            auth,
            subscription: runtime.subscription.clone(),
            unsubscription: runtime.unsubscription.clone(),
        })
    }

    /// Resolves `{{__RUNTIME_SECRET_{path}__}}` placeholders back to their actual values.
    pub async fn resolve(&self, runtime: DataplaneRuntime) -> DataplaneRuntime {
        let mut value = match serde_json::to_value(&runtime) {
            Ok(v) => v,
            Err(_) => return runtime,
        };

        let json_str = value.to_string();
        let keys: HashSet<String> = template_runtime_secret_regex()
            .captures_iter(&json_str)
            .map(|cap| cap[1].to_string())
            .collect();

        if keys.is_empty() {
            return runtime;
        }

        let fetches: Vec<_> = keys
            .iter()
            .map(|k| async move { (k.clone(), self.fetch(k).await) })
            .collect();
        let results = futures_util::future::join_all(fetches).await;
        let cache: HashMap<String, serde_json::Value> = results
            .into_iter()
            .filter_map(|(k, v)| v.map(|val| (k, val)))
            .collect();

        Self::substitute(&mut value, &cache);

        serde_json::from_value(value).unwrap_or(runtime)
    }

    /// Resolves `{{__RUNTIME_SECRET_{path}__}}` placeholders using a `KeystoreLookup`.
    ///
    /// Identical to [`resolve`](Self::resolve) but fetches via `KeystoreLookup::get_secret`
    /// rather than `SecretStore::read` directly.  Use this in the proxy, where the same
    /// `KeystoreClientImpl` instance that the driver factory uses must be shared so that
    /// static connector secrets (e.g. `/ecostars/api_key`) are reachable.
    pub async fn resolve_with_lookup(
        runtime: DataplaneRuntime,
        lookup: &Arc<dyn KeystoreLookup>,
    ) -> DataplaneRuntime {
        let mut value = match serde_json::to_value(&runtime) {
            Ok(v) => v,
            Err(_) => return runtime,
        };

        let json_str = value.to_string();
        let keys: HashSet<String> = template_runtime_secret_regex()
            .captures_iter(&json_str)
            .map(|cap| cap[1].to_string())
            .collect();

        if keys.is_empty() {
            return runtime;
        }

        let fetches: Vec<_> = keys
            .iter()
            .map(|k| {
                let k = k.clone();
                let lookup = lookup.clone();
                async move { (k.clone(), lookup.get_secret(&k).await) }
            })
            .collect();
        let results = futures_util::future::join_all(fetches).await;
        let cache: HashMap<String, serde_json::Value> = results
            .into_iter()
            .filter_map(|(k, v)| v.map(|val| (k, val)))
            .collect();

        Self::substitute(&mut value, &cache);

        serde_json::from_value(value).unwrap_or(runtime)
    }

    /// Deletes all vault secrets associated with a transfer. Call on termination.
    pub async fn cleanup(&self, transfer_id: &str) -> Outcome<()> {
        let prefix = Self::path_prefix(transfer_id);
        let key_prefix = KeyPrefix::new(prefix);
        let entries = self.store.list(&key_prefix).await?;
        let deletes: Vec<_> = entries
            .iter()
            .map(|entry| self.store.delete(&entry.metadata.key))
            .collect();
        for result in futures_util::future::join_all(deletes).await {
            if let Err(e) = result {
                tracing::warn!(error = %e, "vault cleanup: failed to delete secret");
            }
        }
        Ok(())
    }

    async fn upsert(&self, path: &str, value: serde_json::Value) -> Outcome<()> {
        let k = Key::new(path)?;
        self.store.upsert(&k, SecretValue::new(value)).await
    }

    async fn fetch(&self, path: &str) -> Option<serde_json::Value> {
        let k = Key::new(path).ok()?;
        self.store
            .read(&k)
            .await
            .ok()
            .map(|e| e.value.expose().clone())
    }

    fn placeholder(path: &str) -> String {
        format!("{{{{__RUNTIME_SECRET_{{{path}}}__}}}}")
    }

    /// `urn:dataplane-transfer:abc-123` → `/runtime/abc-123`
    fn path_prefix(transfer_id: &str) -> String {
        let id_part = transfer_id.rsplit(':').next().unwrap_or(transfer_id);
        format!("/runtime/{}", id_part)
    }

    fn substitute(value: &mut serde_json::Value, cache: &HashMap<String, serde_json::Value>) {
        match value {
            serde_json::Value::String(s) => {
                if let Some(resolved) = Self::resolve_string(s, cache) {
                    *value = resolved;
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    Self::substitute(v, cache);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    Self::substitute(v, cache);
                }
            }
            _ => {}
        }
    }

    fn resolve_string(
        raw: &str,
        cache: &HashMap<String, serde_json::Value>,
    ) -> Option<serde_json::Value> {
        let re = template_runtime_secret_regex();
        let caps = re.captures(raw)?;
        // Exact match: preserve the JSON type of the stored value.
        if caps.get(0)?.as_str() == raw {
            return cache.get(&caps[1]).cloned();
        }
        // Interpolated: stringify each match.
        let mut result = raw.to_string();
        let mut changed = false;
        for cap in re.captures_iter(raw) {
            if let Some(val) = cache.get(&cap[1]) {
                let s = match val {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                result = result.replace(&cap[0], &s);
                changed = true;
            }
        }
        if changed {
            Some(serde_json::Value::String(result))
        } else {
            None
        }
    }
}
