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

use crate::engine::dataplane_drivers::proxy::http::HttpProxyDriver;
use crate::engine::dataplane_drivers::DriverPubSubTrait;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxyIngress, HTTP_LISTENER_PATH,
};
use crate::engine::dataplane_manager::dataplane_runtime::{DataplaneRuntime, RuntimeSecretVault};
use crate::errors::DataplaneError;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use connector::entities::parameters::TemplateMapString;
use connector::{
    HttpSpec, InteractionConfig, KeystoreLookup, ProtocolSpec, RuntimeParametersResolver,
    TemplateVecString,
};
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Arc;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

pub struct HttpPubSubscriber {
    keystore: Option<Arc<dyn KeystoreLookup>>,
}

impl std::fmt::Debug for HttpPubSubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpPubSubscriber")
            .field("keystore", &self.keystore.is_some())
            .finish()
    }
}

impl HttpPubSubscriber {
    pub fn new(keystore: Option<Arc<dyn KeystoreLookup>>) -> Self {
        Self { keystore }
    }

    /// Builds the relative ingress path exposed to `{{__RUNTIME_INGRESS__}}`, embedding the
    /// per-transfer capability query parameter (`?ingress_token=...`) when configured on the
    /// listener so external webhook callbacks authenticate automatically.
    fn runtime_ingress_path(context: &DataplaneContext) -> String {
        if let Some(proxy) = context.proxy() {
            if let DataplaneProxyIngress::HttpListener { path, token, .. } = proxy.ingress() {
                return match token.as_deref().filter(|t| !t.is_empty()) {
                    Some(tok) => format!("{path}?ingress_token={tok}"),
                    None => path.clone(),
                };
            }
        }
        let dp = &context.dataplane_process().inner.id;
        format!("{HTTP_LISTENER_PATH}{dp}")
    }

    /// Resolves `{{__RUNTIME_SECRET_{...}__}}` placeholders inside `context.runtime()` when a
    /// keystore lookup is attached.
    async fn resolved_runtime(&self, context: &DataplaneContext, user_id: &str) -> DataplaneRuntime {
        let rt = context.runtime().cloned().unwrap_or_default();
        match &self.keystore {
            Some(ks) => RuntimeSecretVault::resolve_with_lookup(rt, ks, user_id).await,
            None => rt,
        }
    }

    /// Builds the outbound URL (appending query-param API keys if any) and merged `HeaderMap`
    /// (spec headers overridden by resolved auth headers) across all 5 `AuthenticationConfig`
    /// variants (`NoAuth`, `BearerToken`, `BasicAuth`, `ApiKey`, `OAuth2`).
    fn prepare_request(
        spec: &HttpSpec,
        runtime: &DataplaneRuntime,
        default_method: &str,
    ) -> Outcome<(String, String, Option<HeaderMap>, Option<Value>)> {
        let mut url = spec.url_template.clone();
        let method = match &spec.method {
            TemplateVecString::Value(v) => v
                .first()
                .map(|s| s.to_uppercase())
                .unwrap_or_else(|| default_method.to_string()),
            TemplateVecString::Template(m) => m.to_uppercase(),
        };
        let body: Option<Value> = spec
            .body_template
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));

        let (auth_headers, query_params) = HttpProxyDriver::build_auth_artifacts(&runtime.auth)?;
        if !query_params.is_empty() {
            let sep = if url.contains('?') { '&' } else { '?' };
            let qs = query_params
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        urlencoding::encode(k),
                        urlencoding::encode(v)
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            url.push(sep);
            url.push_str(&qs);
        }

        let mut headers = HeaderMap::new();
        if let Some(TemplateMapString::Value(spec_headers)) = &spec.headers {
            for (k, v) in spec_headers {
                let name = HeaderName::from_str(k).map_err(|e| {
                    DataplaneError::InvalidHeaderValue {
                        header: k.clone(),
                        reason: e.to_string(),
                    }
                })?;
                let val = HeaderValue::from_str(v).map_err(|e| {
                    DataplaneError::InvalidHeaderValue {
                        header: k.clone(),
                        reason: e.to_string(),
                    }
                })?;
                headers.insert(name, val);
            }
        }
        for (name, val) in auth_headers.iter() {
            headers.insert(name.clone(), val.clone());
        }

        let headers_opt = if headers.is_empty() {
            None
        } else {
            Some(headers)
        };
        Ok((method, url, headers_opt, body))
    }

    async fn execute_http(
        method: &str,
        url: &str,
        headers: Option<HeaderMap>,
        body: Option<Value>,
    ) -> Outcome<Value> {
        let safe_url = Self::redact_url(url);
        match method {
            "POST" => {
                let b = body.unwrap_or_else(|| json!({}));
                http_client()
                    .post_json(url, headers, &b)
                    .await
                    .map_err(|e| {
                        DataplaneError::PubSubRequestFailed {
                            method: "POST".to_string(),
                            url: safe_url,
                            reason: e.to_string(),
                        }
                        .into()
                    })
            }
            "PUT" => {
                let b = body.unwrap_or_else(|| json!({}));
                http_client()
                    .put_json(url, headers, &b)
                    .await
                    .map_err(|e| {
                        DataplaneError::PubSubRequestFailed {
                            method: "PUT".to_string(),
                            url: safe_url,
                            reason: e.to_string(),
                        }
                        .into()
                    })
            }
            "DELETE" => {
                http_client()
                    .delete_ok(url, headers)
                    .await
                    .map_err(|e| DataplaneError::PubSubRequestFailed {
                        method: "DELETE".to_string(),
                        url: safe_url,
                        reason: e.to_string(),
                    })?;
                Ok(Value::Null)
            }
            "GET" => http_client().get_json(url, headers).await.map_err(|e| {
                DataplaneError::PubSubRequestFailed {
                    method: "GET".to_string(),
                    url: safe_url,
                    reason: e.to_string(),
                }
                .into()
            }),
            other => Err(DataplaneError::UnsupportedProtocol {
                protocol: format!("HTTP method {other}"),
            }
            .into()),
        }
    }

    fn redact_url(raw: &str) -> String {
        let Ok(mut parsed) = url::Url::parse(raw) else {
            return raw.to_string();
        };
        if parsed.query().is_none() {
            return raw.to_string();
        }
        let pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .map(|(k, v)| {
                let lower = k.to_ascii_lowercase();
                if lower.contains("key")
                    || lower.contains("token")
                    || lower.contains("secret")
                    || lower.contains("password")
                    || lower.contains("auth")
                {
                    (k.into_owned(), "[REDACTED]".to_string())
                } else {
                    (k.into_owned(), v.into_owned())
                }
            })
            .collect();
        parsed.query_pairs_mut().clear().extend_pairs(pairs);
        parsed.into()
    }
}

#[async_trait::async_trait]
impl DriverPubSubTrait for HttpPubSubscriber {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "data-endpoint"))]
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context.connector_instance().ok_or_else(|| {
            DataplaneError::PubSubConnectorNotAvailable {
                operation: "subscribe".to_string(),
            }
        })?;

        let ingress_url = Self::runtime_ingress_path(context);
        let resolved_rt = self.resolved_runtime(context, &connector.user_id).await;
        let runtime_value = serde_json::to_value(&resolved_rt)?;
        let mut resolver = RuntimeParametersResolver::new(connector, &runtime_value)
            .with_ingress(Some(&ingress_url));
        if let Some(ks) = &self.keystore {
            resolver = resolver.with_keystore(ks.clone(), &connector.user_id);
        }
        let resolved_connector = resolver.resolve().await?;
        let push_lifecycle = match &resolved_connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => {
                return Err(DataplaneError::WrongInteractionType {
                    expected: "Push".to_string(),
                    found: "other".to_string(),
                }
                .into())
            }
        };
        let http_spec = match &push_lifecycle.subscribe {
            ProtocolSpec::Http(spec) => spec,
            _ => {
                return Err(DataplaneError::UnsupportedProtocol {
                    protocol: "non-HTTP subscribe".to_string(),
                }
                .into())
            }
        };

        let (method, url, headers, body) = Self::prepare_request(http_spec, &resolved_rt, "POST")?;
        let response = Self::execute_http(&method, &url, headers, body).await?;

        let mut ctx = context.clone();
        let mut runtime = ctx.runtime().cloned().unwrap_or_default();
        runtime.subscription = response;
        ctx.set_runtime(runtime);
        Ok(ctx)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "data-endpoint"))]
    async fn unsubscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context.connector_instance().ok_or_else(|| {
            DataplaneError::PubSubConnectorNotAvailable {
                operation: "unsubscribe".to_string(),
            }
        })?;
        match &connector.interaction {
            InteractionConfig::Push(_) => {}
            _ => {
                return Err(DataplaneError::WrongInteractionType {
                    expected: "Push".to_string(),
                    found: "other".to_string(),
                }
                .into())
            }
        };

        let ingress_url = Self::runtime_ingress_path(context);
        let resolved_rt = self.resolved_runtime(context, &connector.user_id).await;
        let runtime_value = serde_json::to_value(&resolved_rt)?;
        let mut resolver = RuntimeParametersResolver::new(connector, &runtime_value)
            .with_ingress(Some(&ingress_url));
        if let Some(ks) = &self.keystore {
            resolver = resolver.with_keystore(ks.clone(), &connector.user_id);
        }
        let current_instance = resolver.resolve().await?;

        let resolved_push = match &current_instance.interaction {
            InteractionConfig::Push(p) => p,
            _ => {
                return Err(DataplaneError::WrongInteractionType {
                    expected: "Push".to_string(),
                    found: "other".to_string(),
                }
                .into())
            }
        };
        // `unsubscribe` is optional in `PushLifecycle`: when `None`, nothing needs tearing down.
        let Some(unsub_spec) = &resolved_push.unsubscribe else {
            return Ok(context.clone());
        };
        let ProtocolSpec::Http(resolved_http) = unsub_spec else {
            return Err(DataplaneError::UnsupportedProtocol {
                protocol: "non-HTTP unsubscribe".to_string(),
            }
            .into());
        };

        let (method, url, headers, body) =
            Self::prepare_request(resolved_http, &resolved_rt, "DELETE")?;
        let response = Self::execute_http(&method, &url, headers, body).await?;

        let mut ctx = context.clone();
        let mut runtime = ctx.runtime().cloned().unwrap_or_default();
        runtime.unsubscription = response;
        ctx.set_runtime(runtime);
        Ok(ctx)
    }
}
