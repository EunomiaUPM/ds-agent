use crate::entities::dataplane_drivers::DriverPubSubTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
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
}

#[async_trait::async_trait]
impl DriverPubSubTrait for HttpPubSubscriber {
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        // extract subscribe spec
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("No connector instance for PUSH subscribe", None))?;
        let push_lifecycle = match &connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => return Err(Errors::crazy("Connector interaction is not PUSH", None)),
        };
        let http_spec = match &push_lifecycle.subscribe {
            ProtocolSpec::Http(spec) => spec,
            _ => return Err(Errors::crazy("Only HTTP subscribe is supported", None)),
        };

        // perform subscription
        let body: Option<Value> = http_spec
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));
        let url = http_spec.url_template.clone();
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
        // no unsubscribe spec → return context untouched
        if push_lifecycle.unsubscribe.is_none() {
            return Ok(context.clone());
        }

        // resolve RUNTIME_JSON_{*} placeholders against current runtime state
        let runtime_value = serde_json::to_value(context.runtime().cloned().unwrap_or_default())?;
        let current_instance =
            RuntimeParametersResolver::new(connector, &runtime_value).resolve()?;

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
            TemplateVecString::Template(_) => "DELETE".to_string(),
        };
        let body: Option<Value> = resolved_http
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));

        // perform unsubscription
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
