use serde::{Deserialize, Serialize};

pub const HTTP_LISTENER_PATH: &str = "/dataplane/proxy/";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DataplaneProxyIngress {
    NoOp,
    HttpListener {
        path: String,
        token_type: Option<String>,
        token: Option<String>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DataplaneProxy {
    pub(crate) ingress: DataplaneProxyIngress,
    pub(crate) egress: DataplaneProxyEgress,
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
