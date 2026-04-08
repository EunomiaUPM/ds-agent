use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum DataplaneProxyIngress {
    NoOp,
    HttpListener {
        path: String,
        token_type: Option<String>,
        token: Option<String>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DataplaneProxyEgress {
    NoOp,
    HttpProxy {
        path: String,
        token_type: Option<String>,
        token: Option<String>,
    },
    DataClient {
        path: String,
        token_type: Option<String>,
        token: Option<String>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataplaneProxy {
    ingress: DataplaneProxyIngress,
    egress: DataplaneProxyEgress,
}

impl DataplaneProxy {
    pub fn new() -> Self {
        Self {
            ingress: DataplaneProxyIngress::NoOp,
            egress: DataplaneProxyEgress::NoOp,
        }
    }

    pub fn ingress(&self) -> &DataplaneProxyIngress {
        &self.ingress
    }
    pub fn egress(&self) -> &DataplaneProxyEgress {
        &self.egress
    }
    pub fn set_ingress(&mut self, ingress: DataplaneProxyIngress) -> &mut DataplaneProxy {
        self.ingress = ingress;
        self
    }

    pub fn set_egress(&mut self, egress: DataplaneProxyEgress) -> &mut DataplaneProxy {
        self.egress = egress;
        self
    }
}
