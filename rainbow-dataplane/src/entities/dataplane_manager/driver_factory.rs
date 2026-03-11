use crate::entities::dataplane_manager::config_builder::IngressConfig;
use crate::entities::dataplane_transfers::{DataplaneTransferDto, InteractionMode, TransferRole};
use rainbow_common::http_client::HttpClient;
use rainbow_connector::{
    ConnectorInstanceDto, InteractionConfig, ProtocolSpec, TemplateParametersResolver,
    TemplateResolverVisitor,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use urn::Urn;

// ─── System runtime context ───
//
// Computed at subscribe/unsubscribe time from the active dataplane process and node config.
// Keys injected into the template resolver:
//   RUNTIME_URN                  → the dataplane transfer URN
//   RUNTIME_OWN_URL              → proxy base URL (from config)
//   RUNTIME_OWN_URL_DOCKERIZED   → Docker-reachable proxy URL (localhost → host.docker.internal)
//   RUNTIME_CB_URL               → RUNTIME_OWN_URL + ingress path (full callback URL)
//   RUNTIME_CB_URL_DOCKERIZED    → RUNTIME_OWN_URL_DOCKERIZED + ingress path
//
// Additional keys via response contexts (resolved inline via jq):
//   RUNTIME_SUB_RESPONSE_{jq}   → jq expression applied to subscribe response (flow_control)
//   RUNTIME_AUTH_RESPONSE_{jq}  → jq expression applied to auth response (future)
//
// Usage in templates:
//   {{__RUNTIME_CB_URL_DOCKERIZED__}}          → full dockerized callback URL
//   {{__RUNTIME_SUB_RESPONSE_{.id}__}}         → .id from subscribe response
//   {{__RUNTIME_SUB_RESPONSE_{.data.token}__}} → nested field from subscribe response

pub struct SysRuntimeContext {
    pub transfer_id: Urn,
    pub proxy_base_url: String,
    pub ingress_path: String,
    /// Subscribe response stored in flow_control after perform_subscribe.
    /// Exposed at unsubscribe time via RUNTIME_SUB_RESPONSE_{jq} placeholders.
    pub subscription_state: Option<Value>,
}

impl SysRuntimeContext {
    pub fn to_params(&self) -> HashMap<String, Value> {
        let cb_url = format!("{}{}", self.proxy_base_url, self.ingress_path);
        let proxy_dockerized = self
            .proxy_base_url
            .replace("localhost", "host.docker.internal")
            .replace("127.0.0.1", "host.docker.internal");
        let cb_url_dockerized = format!("{}{}", proxy_dockerized, self.ingress_path);

        let mut map = HashMap::new();
        map.insert("RUNTIME_URN".to_string(), json!(self.transfer_id.to_string()));
        map.insert("RUNTIME_OWN_URL".to_string(), json!(self.proxy_base_url));
        map.insert("RUNTIME_OWN_URL_DOCKERIZED".to_string(), json!(proxy_dockerized));
        map.insert("RUNTIME_CB_URL".to_string(), json!(cb_url));
        map.insert("RUNTIME_CB_URL_DOCKERIZED".to_string(), json!(cb_url_dockerized));
        map
    }
}

// ─── HTTP subscribe lifecycle driver ───

pub struct HttpSubscribeLifecycle {
    http_client: Arc<HttpClient>,
    sys_context: SysRuntimeContext,
}

impl HttpSubscribeLifecycle {
    pub fn new(http_client: Arc<HttpClient>, sys_context: SysRuntimeContext) -> Self {
        Self { http_client, sys_context }
    }
}

#[async_trait::async_trait]
impl LifeCycleActionTrait for HttpSubscribeLifecycle {
    async fn perform_subscribe(
        &self,
        connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<Value> {
        let connector =
            connector.ok_or_else(|| anyhow::anyhow!("No connector instance for PUSH subscribe"))?;
        let push = match &connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => return Err(anyhow::anyhow!("Connector interaction is not PUSH")),
        };

        // Clone subscribe spec and apply RUNTIME_* params (no SUB context yet at subscribe time).
        let mut subscribe_spec = push.subscribe.clone();
        let runtime_params = self.sys_context.to_params();
        let mut resolver = TemplateParametersResolver::new(&runtime_params);
        TemplateResolverVisitor::new(&mut resolver).apply_protocol(&mut subscribe_spec)?;

        let http_spec = match &subscribe_spec {
            ProtocolSpec::Http(spec) => spec,
            _ => return Err(anyhow::anyhow!("Only HTTP subscribe is supported")),
        };

        let body: Option<Value> = http_spec
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));

        let url = http_spec.url_template.clone();
        let response: Value = if let Some(body) = body {
            self.http_client
                .post_json(&url, &body)
                .await
                .map_err(|e| anyhow::anyhow!("Subscribe POST failed: {}", e))?
        } else {
            self.http_client
                .post_json(&url, &json!({}))
                .await
                .map_err(|e| anyhow::anyhow!("Subscribe POST (no body) failed: {}", e))?
        };
        Ok(response)
    }

    async fn perform_unsubscribe(
        &self,
        connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<()> {
        let connector = connector
            .ok_or_else(|| anyhow::anyhow!("No connector instance for PUSH unsubscribe"))?;
        let push = match &connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => return Err(anyhow::anyhow!("Connector interaction is not PUSH")),
        };

        let mut unsubscribe_spec = match &push.unsubscribe {
            Some(spec) => spec.clone(),
            None => return Ok(()),
        };

        // Second-pass resolver: RUNTIME_* params + SUB response context for RUNTIME_SUB_RESPONSE_{jq}
        let runtime_params = self.sys_context.to_params();
        let sub_state = self.sys_context.subscription_state.clone().unwrap_or(Value::Null);
        let mut resolver = TemplateParametersResolver::new(&runtime_params)
            .with_response_context("SUB", sub_state);
        TemplateResolverVisitor::new(&mut resolver).apply_protocol(&mut unsubscribe_spec)?;

        let http_spec = match &unsubscribe_spec {
            ProtocolSpec::Http(spec) => spec,
            _ => return Err(anyhow::anyhow!("Only HTTP unsubscribe is supported")),
        };

        let body: Option<Value> = http_spec
            .body_template
            .as_ref()
            .map(|s| serde_json::from_str(s).unwrap_or_else(|_| json!(s)));

        let url = http_spec.url_template.clone();
        if let Some(body) = body {
            let _: Value = self
                .http_client
                .post_json(&url, &body)
                .await
                .map_err(|e| anyhow::anyhow!("Unsubscribe POST failed: {}", e))?;
        } else {
            let _: Value = self
                .http_client
                .post_json(&url, &json!({}))
                .await
                .map_err(|e| anyhow::anyhow!("Unsubscribe POST (no body) failed: {}", e))?;
        }
        Ok(())
    }
}

// ─── Factory ───

pub struct DataplaneDriverFactory {
    proxy_base_url: String,
    http_client: Arc<HttpClient>,
}

impl DataplaneDriverFactory {
    pub fn new(proxy_base_url: String, http_client: Arc<HttpClient>) -> Self {
        Self { proxy_base_url, http_client }
    }

    pub fn create_driver(
        &self,
        process: &DataplaneTransferDto,
        connector_instance_dto: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<DataplaneDriver> {
        let role = process.inner.role.clone();
        let interaction_mode = process.inner.interaction_mode.clone();

        let auth_driver: Arc<dyn AuthActionTrait> = Arc::new(NoOpAuth::new());

        let lifecycle_driver: Arc<dyn LifeCycleActionTrait> = match (&role, &interaction_mode) {
            (TransferRole::Provider, InteractionMode::Push) => {
                let ingress_path = self.extract_ingress_path(process)?;
                let transfer_id: Urn = process.inner.id.parse()?;
                let sys_context = SysRuntimeContext {
                    transfer_id,
                    proxy_base_url: self.proxy_base_url.clone(),
                    ingress_path,
                    subscription_state: process.inner.flow_control.clone(),
                };
                Arc::new(HttpSubscribeLifecycle::new(self.http_client.clone(), sys_context))
            }
            _ => Arc::new(NoOpLifecycle::new()),
        };

        Ok(DataplaneDriver { auth_driver, lifecycle_driver })
    }

    fn extract_ingress_path(&self, process: &DataplaneTransferDto) -> anyhow::Result<String> {
        let ingress: IngressConfig =
            serde_json::from_value(process.inner.ingress_config.clone())
                .map_err(|e| anyhow::anyhow!("Failed to parse ingress_config: {}", e))?;
        match ingress {
            IngressConfig::HttpListener { path } => Ok(path),
            IngressConfig::Connector { .. } => {
                Err(anyhow::anyhow!("Cannot compute callback URL for Connector ingress"))
            }
        }
    }
}

pub struct DataplaneDriver {
    pub auth_driver: Arc<dyn AuthActionTrait>,
    pub lifecycle_driver: Arc<dyn LifeCycleActionTrait>,
}

#[async_trait::async_trait]
pub trait AuthActionTrait: Send + Sync {
    async fn perform_auth(&self, connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()>;
}

pub struct NoOpAuth {}

impl NoOpAuth {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl AuthActionTrait for NoOpAuth {
    async fn perform_auth(&self, _connector: Option<&ConnectorInstanceDto>) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait LifeCycleActionTrait: Send + Sync {
    /// Performs subscription. Returns the response body (stored in flow_control for later use).
    async fn perform_subscribe(
        &self,
        connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<Value>;
    async fn perform_unsubscribe(
        &self,
        connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<()>;
}

pub struct NoOpLifecycle {}

impl NoOpLifecycle {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl LifeCycleActionTrait for NoOpLifecycle {
    async fn perform_subscribe(
        &self,
        _connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<Value> {
        Ok(Value::Null)
    }

    async fn perform_unsubscribe(
        &self,
        _connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
