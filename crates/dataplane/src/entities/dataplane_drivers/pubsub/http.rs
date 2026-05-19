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

use crate::entities::dataplane_drivers::DriverPubSubTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_proxy::HTTP_LISTENER_PATH;
use crate::entities::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
use common::http_client::HttpClient;
use connector::{InteractionConfig, ProtocolSpec, RuntimeParametersResolver, TemplateVecString};
use serde_json::{json, Value};
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

#[derive(Debug)]
pub struct HttpPubSubscriber {
    http_client: HttpClient,
}

impl HttpPubSubscriber {
    pub fn new() -> Self {
        let http_client = HttpClient::new(1, 1);
        Self { http_client }
    }

    /// Reads resolved credentials from the context runtime and configures the HTTP
    /// client auth token for the next request.
    ///
    /// Only `BearerToken` and `OAuth2` are supported here — `HttpClient` sends an
    /// `Authorization: Bearer <token>` header. Other credential types (BasicAuth,
    /// ApiKey) cannot be injected through this client and are left as no-ops.
    async fn apply_auth(&self, context: &DataplaneContext) {
        let Some(runtime) = context.runtime() else {
            return;
        };
        match &runtime.auth {
            ResolvedAuthCredentials::BearerToken { token } => {
                self.http_client.set_auth_token(token.clone()).await;
            }
            ResolvedAuthCredentials::OAuth2 { access_token, .. } => {
                self.http_client.set_auth_token(access_token.clone()).await;
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl DriverPubSubTrait for HttpPubSubscriber {
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        // extract subscribe spec
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("No connector instance for PUSH subscribe", None))?;

        let dp = &context.dataplane_process().inner.id;
        let ingress_url = format!("{}{}", HTTP_LISTENER_PATH, dp);
        let runtime_value = serde_json::to_value(context.runtime().cloned().unwrap_or_default())?;
        let resolved_connector = RuntimeParametersResolver::new(connector, &runtime_value)
            .with_ingress(Some(ingress_url))
            .resolve()?;
        let push_lifecycle = match &resolved_connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => {
                return Err(Errors::crazy(
                    "Resolved connector interaction is not PUSH",
                    None,
                ))
            }
        };
        let http_spec = match &push_lifecycle.subscribe {
            ProtocolSpec::Http(spec) => spec,
            _ => return Err(Errors::crazy("Only HTTP subscribe is supported", None)),
        };

        let body: Option<Value> = http_spec
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));
        let url = http_spec.url_template.clone();
        self.apply_auth(context).await;
        let response: Value = {
            let b = body.unwrap_or(json!({}));
            self.http_client.post_json(&url, &b).await.map_err(|e| {
                Errors::crazy(format!("Subscribe POST failed: {}", e), Some(Box::new(e)))
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
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("No connector instance for PUSH unsubscribe", None))?;
        let push_lifecycle = match &connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => return Err(Errors::crazy("Connector interaction is not PUSH", None)),
        };

        // resolve placeholders against current runtime state and ingress address
        let ingress_url = context
            .forward_dataplane_address()
            .map(|a| a.endpoint.as_str());
        let runtime_value = serde_json::to_value(context.runtime().cloned().unwrap_or_default())?;
        let current_instance = RuntimeParametersResolver::new(connector, &runtime_value)
            .with_ingress(ingress_url)
            .resolve()?;

        // re-extract the resolved unsubscribe spec
        let resolved_push = match &current_instance.interaction {
            InteractionConfig::Push(p) => p,
            _ => {
                return Err(Errors::crazy(
                    "Resolved connector interaction is not PUSH",
                    None,
                ))
            }
        };
        let resolved_http = match &resolved_push.unsubscribe {
            Some(ProtocolSpec::Http(s)) => s,
            _ => return Err(Errors::crazy("Resolved unsubscribe spec is not HTTP", None)),
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
        self.apply_auth(context).await;
        let response: Value = match method.as_str() {
            "DELETE" => {
                self.http_client.delete::<()>(&url).await.map_err(|e| {
                    Errors::crazy(
                        format!("Unsubscribe DELETE failed: {}", e),
                        Some(Box::new(e)),
                    )
                })?;
                Value::Null
            }
            "POST" => {
                let b = body.unwrap_or(json!({}));
                self.http_client.post_json(&url, &b).await.map_err(|e| {
                    Errors::crazy(format!("Unsubscribe POST failed: {}", e), Some(Box::new(e)))
                })?
            }
            "PUT" => {
                let b = body.unwrap_or(json!({}));
                self.http_client.put_json(&url, &b).await.map_err(|e| {
                    Errors::crazy(format!("Unsubscribe PUT failed: {}", e), Some(Box::new(e)))
                })?
            }
            "GET" => self.http_client.get_json(&url).await.map_err(|e| {
                Errors::crazy(format!("Unsubscribe GET failed: {}", e), Some(Box::new(e)))
            })?,
            other => {
                return Err(Errors::crazy(
                    format!("Unsupported unsubscribe HTTP method: {}", other),
                    None,
                ))
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
