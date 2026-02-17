use crate::entities::dataplane_transfers::DataplaneTransferDto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IngressConfig {
    HttpProxy { url: String },
    HttpListener { path: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EgressConfig {
    HttpDispatcher { url: String },
    InternalConnecto { path: String },
}

pub struct DataplaneConfigBuilder {
    pub ingress: serde_json::Value,
    pub egress: serde_json::Value,
}
impl DataplaneConfigBuilder {
    pub fn from_process(process: &DataplaneTransferDto) -> Self {
        Self {
            ingress: process.inner.ingress_config.clone(),
            egress: process.inner.egress_config.clone(),
        }
    }

    pub fn apply_ingress<T: Serialize>(&mut self, config: T) {
        self.ingress = serde_json::to_value(config).unwrap_or(serde_json::Value::Null);
    }

    pub fn apply_egress<T: Serialize>(&mut self, config: T) {
        self.egress = serde_json::to_value(config).unwrap_or(serde_json::Value::Null);
    }
}
