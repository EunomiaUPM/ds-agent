use crate::DataplaneAddress;
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

impl Into<DataplaneAddress> for DataplaneProxyEgress {
    fn into(self) -> DataplaneAddress {
        match self {
            DataplaneProxyEgress::NoOp => DataplaneAddress {
                endpoint_type: "".to_string(),
                endpoint: "".to_string(),
                authorization_type: None,
                authorization: None,
            },
            DataplaneProxyEgress::HttpProxy {
                path,
                token,
                token_type,
            } => DataplaneAddress {
                endpoint_type: "HTTP".to_string(),
                endpoint: path,
                authorization_type: token_type,
                authorization: token,
            },
            DataplaneProxyEgress::DataClient {
                path,
                token,
                token_type,
            } => DataplaneAddress {
                endpoint_type: "HTTP".to_string(),
                endpoint: path,
                authorization_type: token_type,
                authorization: token,
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DataplaneProxy {
    pub(crate) ingress: DataplaneProxyIngress,
    pub(crate) egress: DataplaneProxyEgress,
}

impl Default for DataplaneProxy {
    fn default() -> Self {
        Self::new()
    }
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
