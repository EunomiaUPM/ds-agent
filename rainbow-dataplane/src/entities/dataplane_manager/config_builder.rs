use crate::entities::dataplane_transfers::{DataplaneTransferDto, TransferRole};
use rainbow_connector::{ConnectorInstanceDto, InteractionConfig, ProtocolSpec};
use serde::{Deserialize, Serialize};

// ─── Ingress: how data enters this node's proxy ───
//
// This is the "front door": the endpoint that receives requests/streams.
//   - PULL Provider: HttpListener (auto path — becomes the DataAddress sent to consumer)
//   - PULL Consumer: HttpListener (auto path — exposed to data client app)
//   - PUSH Provider: HttpListener (auto path — backend calls this callback to push data;
//                   perform_subscribe tells the backend to use this URL as push target)
//   - PUSH Consumer: HttpListener (auto path — ingest URL sent to provider)

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IngressConfig {
    /// Auto-generated proxy listener path (exposed to callers)
    HttpListener { path: String },
    /// Provider PUSH: connector's subscribe callback receives data from backend
    Connector { spec: ProtocolSpec },
}

// ─── Egress: how data exits this node's proxy ───
//
// This is the "back door": where the proxy forwards data to.
//   - PULL Provider: Connector data_access (final system URL)
//   - PULL Consumer: Provider's DataAddress (filled later from DSP TransferStart)
//   - PUSH Provider: Consumer's ingest URL (filled later from TransferRequest DataAddress)
//   - PUSH Consumer: Data client's DataAddress (where to deliver received data)

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EgressConfig {
    /// Proxy forwards to a remote URL (generic HTTP forward)
    HttpProxy { url: String },
    /// Provider PULL: forward to the final system via connector spec
    Connector { spec: ProtocolSpec },
    /// Forward to a DataAddress endpoint (consumer PUSH → client destination)
    DataAddress {
        endpoint_type: String,
        endpoint: String,
    },
}

// ─── Builder ───

pub struct DataplaneConfigBuilder {
    pub ingress: serde_json::Value,
    pub egress: serde_json::Value,
}

impl DataplaneConfigBuilder {
    /// Load existing config from a persisted process.
    pub fn from_process(process: &DataplaneTransferDto) -> Self {
        Self {
            ingress: process.inner.ingress_config.clone(),
            egress: process.inner.egress_config.clone(),
        }
    }

    /// Build initial config from role + connector at creation time.
    ///
    /// PULL Provider: ingress = HttpListener (auto), egress = Connector (data_access)
    /// PULL Consumer: ingress = HttpListener (auto), egress = null (filled from provider DataAddress)
    /// PUSH Provider: ingress = Connector (subscribe), egress = null (filled from consumer DataAddress)
    /// PUSH Consumer: ingress = HttpListener (auto), egress = null (filled from client DataAddress)
    pub fn from_connector(
        role: &TransferRole,
        connector: Option<&ConnectorInstanceDto>,
        transfer_process_id: &str,
    ) -> Self {
        let auto_path = format!("/{}", transfer_process_id);

        match (role, connector) {
            // ─── PULL Provider ───
            // Ingress: proxy listener (DataAddress sent to consumer)
            // Egress: HTTP URL from connector's data_access (final system)
            (TransferRole::Provider, Some(ci)) if matches!(ci.interaction, InteractionConfig::Pull(_)) => {
                let pull = match &ci.interaction {
                    InteractionConfig::Pull(p) => p,
                    _ => unreachable!(),
                };
                let egress = match &pull.data_access {
                    ProtocolSpec::Http(http) => {
                        to_json(EgressConfig::HttpProxy { url: http.url_template.clone() })
                    }
                    spec => to_json(EgressConfig::Connector { spec: spec.clone() }),
                };
                Self {
                    ingress: to_json(IngressConfig::HttpListener { path: auto_path }),
                    egress,
                }
            }

            // ─── PUSH Provider ───
            // Ingress: HttpListener (auto path — the proxy callback URL that the backend pushes to;
            //          when perform_subscribe runs it tells the backend to push here)
            // Egress: null (filled later from consumer's ingest URL via SetEgress)
            (TransferRole::Provider, Some(ci)) if matches!(ci.interaction, InteractionConfig::Push(_)) => {
                Self {
                    ingress: to_json(IngressConfig::HttpListener { path: auto_path }),
                    egress: serde_json::Value::Null,
                }
            }

            // ─── Consumer (PULL or PUSH) ───
            // Ingress: auto HttpListener (data client or provider pushes here)
            // Egress: null (filled later from DataAddress)
            _ => Self {
                ingress: to_json(IngressConfig::HttpListener { path: auto_path }),
                egress: serde_json::Value::Null,
            },
        }
    }

    pub fn apply_ingress<T: Serialize>(&mut self, config: T) {
        self.ingress = serde_json::to_value(config).unwrap_or(serde_json::Value::Null);
    }

    pub fn apply_egress<T: Serialize>(&mut self, config: T) {
        self.egress = serde_json::to_value(config).unwrap_or(serde_json::Value::Null);
    }
}

fn to_json<T: Serialize>(val: T) -> serde_json::Value {
    serde_json::to_value(val).unwrap_or_default()
}
