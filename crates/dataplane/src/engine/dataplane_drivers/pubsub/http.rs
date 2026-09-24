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

use crate::engine::dataplane_drivers::DriverPubSubTrait;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_proxy::HTTP_LISTENER_PATH;
use crate::engine::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
use crate::errors::DataplaneError;
use axum::http::HeaderMap;
use connector::{
    InteractionConfig, KeystoreLookup, ProtocolSpec, RuntimeParametersResolver, TemplateVecString,
};
use serde_json::{json, Value};
use std::sync::Arc;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::utils::{bearer_headers, http_client};

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

    /// Bearer headers from the context's resolved credentials, sent with this request only.
    ///
    /// Only `BearerToken` and `OAuth2` are supported here; other credential types
    /// (BasicAuth, ApiKey) send no `Authorization` header.
    fn auth_headers(context: &DataplaneContext) -> Outcome<Option<HeaderMap>> {
        let token = match context.runtime().map(|r| &r.auth) {
            Some(ResolvedAuthCredentials::BearerToken { token }) => token,
            Some(ResolvedAuthCredentials::OAuth2 { access_token, .. }) => access_token,
            _ => return Ok(None),
        };
        bearer_headers(token).map(Some)
    }
}

#[async_trait::async_trait]
impl DriverPubSubTrait for HttpPubSubscriber {
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        // extract subscribe spec
        let connector = context.connector_instance().ok_or_else(|| {
            DataplaneError::PubSubConnectorNotAvailable {
                operation: "subscribe".to_string(),
            }
        })?;

        let dp = &context.dataplane_process().inner.id;
        let ingress_url = format!("{}{}", HTTP_LISTENER_PATH, dp);
        let runtime_value = serde_json::to_value(context.runtime().cloned().unwrap_or_default())?;
        let mut resolver = RuntimeParametersResolver::new(connector, &runtime_value)
            .with_ingress(Some(ingress_url));
        if let Some(ks) = &self.keystore {
            resolver =
                resolver.with_keystore(ks.clone(), &context.dataplane_process().inner.tenant_id);
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

        let body: Option<Value> = http_spec
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));
        let url = http_spec.url_template.clone();
        let headers = Self::auth_headers(context)?;
        let response: Value = {
            let b = body.unwrap_or(json!({}));
            http_client()
                .post_json(&url, headers, &b)
                .await
                .map_err(|e| DataplaneError::PubSubRequestFailed {
                    method: "POST".to_string(),
                    url: url.clone(),
                    reason: e.to_string(),
                })?
        };

        // store subscription info in context
        let mut ctx = context.clone();
        let mut runtime = ctx.runtime().cloned().unwrap_or_default();
        runtime.subscription = response;
        ctx.set_runtime(runtime);
        Ok(ctx)
    }

    async fn unsubscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        // extract unsubscribe spec
        let connector = context.connector_instance().ok_or_else(|| {
            DataplaneError::PubSubConnectorNotAvailable {
                operation: "unsubscribe".to_string(),
            }
        })?;
        let push_lifecycle = match &connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => {
                return Err(DataplaneError::WrongInteractionType {
                    expected: "Push".to_string(),
                    found: "other".to_string(),
                }
                .into())
            }
        };

        // resolve placeholders against current runtime state and ingress address
        let ingress_url = context
            .forward_dataplane_address()
            .map(|a| a.endpoint.as_str());
        let runtime_value = serde_json::to_value(context.runtime().cloned().unwrap_or_default())?;
        let mut resolver =
            RuntimeParametersResolver::new(connector, &runtime_value).with_ingress(ingress_url);
        if let Some(ks) = &self.keystore {
            resolver =
                resolver.with_keystore(ks.clone(), &context.dataplane_process().inner.tenant_id);
        }
        let current_instance = resolver.resolve().await?;

        // re-extract the resolved unsubscribe spec
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
        let resolved_http = match &resolved_push.unsubscribe {
            Some(ProtocolSpec::Http(s)) => s,
            _ => {
                return Err(DataplaneError::UnsupportedProtocol {
                    protocol: "non-HTTP unsubscribe".to_string(),
                }
                .into())
            }
        };

        let url = resolved_http.url_template.clone();
        let method = match &resolved_http.method {
            TemplateVecString::Value(v) => v
                .first()
                .map(|s| s.to_uppercase())
                .unwrap_or_else(|| "DELETE".to_string()),
            TemplateVecString::Template(m) => m.to_string(),
        };
        let body: Option<Value> = resolved_http
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));

        // perform unsubscription
        let headers = Self::auth_headers(context)?;
        let response: Value = match method.as_str() {
            "DELETE" => {
                http_client().delete_ok(&url, headers).await.map_err(|e| {
                    DataplaneError::PubSubRequestFailed {
                        method: "DELETE".to_string(),
                        url: url.clone(),
                        reason: e.to_string(),
                    }
                })?;
                Value::Null
            }
            "POST" => {
                let b = body.unwrap_or(json!({}));
                http_client()
                    .post_json(&url, headers, &b)
                    .await
                    .map_err(|e| DataplaneError::PubSubRequestFailed {
                        method: "POST".to_string(),
                        url: url.clone(),
                        reason: e.to_string(),
                    })?
            }
            "PUT" => {
                let b = body.unwrap_or(json!({}));
                http_client()
                    .put_json(&url, headers, &b)
                    .await
                    .map_err(|e| DataplaneError::PubSubRequestFailed {
                        method: "PUT".to_string(),
                        url: url.clone(),
                        reason: e.to_string(),
                    })?
            }
            "GET" => http_client().get_json(&url, headers).await.map_err(|e| {
                DataplaneError::PubSubRequestFailed {
                    method: "GET".to_string(),
                    url: url.clone(),
                    reason: e.to_string(),
                }
            })?,
            other => {
                return Err(DataplaneError::UnsupportedProtocol {
                    protocol: format!("HTTP method {}", other),
                }
                .into())
            }
        };

        // store unsubscription info in context
        let mut ctx = context.clone();
        let mut runtime = ctx.runtime().cloned().unwrap_or_default();
        runtime.unsubscription = response;
        ctx.set_runtime(runtime);
        Ok(ctx)
    }
}
