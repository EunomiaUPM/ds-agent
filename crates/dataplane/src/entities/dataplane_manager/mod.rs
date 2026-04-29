use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::dsp_common::data_address::{DataAddress, EndpointProperty};
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

pub(crate) mod dataplane_commands;
pub(crate) mod dataplane_context;
pub(crate) mod dataplane_driver_factory;
mod dataplane_handlers_consumer_pull;
mod dataplane_handlers_consumer_push;
mod dataplane_handlers_provider_pull;
mod dataplane_handlers_provider_push;
mod dataplane_handlers_strategy;
pub(crate) mod dataplane_manager;
pub(crate) mod dataplane_proxy;
pub(crate) mod dataplane_runtime;

#[derive(Debug, Clone)]
pub struct DataplaneAddress {
    pub endpoint_type: String,
    pub endpoint: String,
    pub authorization_type: Option<String>,
    pub authorization: Option<String>,
}

impl Into<DataplaneAddress> for DataAddress {
    fn into(self) -> DataplaneAddress {
        DataplaneAddress {
            endpoint_type: self.endpoint_type,
            endpoint: self.endpoint,
            authorization_type: self
                .endpoint_properties
                .iter()
                .find(|p| p.name == "authType")
                .map(|p| p.value.clone()),
            authorization: self
                .endpoint_properties
                .iter()
                .find(|p| p.name == "authorization")
                .map(|p| p.value.clone()),
        }
    }
}

impl Into<DataAddress> for DataplaneAddress {
    fn into(self) -> DataAddress {
        let ep_authorization_type = match self.authorization_type {
            Some(at) => Some(EndpointProperty {
                _type: "EndpointProperty".to_string(),
                name: "authType".to_string(),
                value: at.to_string(),
            }),
            None => None,
        };
        let ep_authorization = match self.authorization {
            Some(at) => Some(EndpointProperty {
                _type: "EndpointProperty".to_string(),
                name: "authorization".to_string(),
                value: at.to_string(),
            }),
            None => None,
        };
        let mut endpoint_properties: Vec<EndpointProperty> = vec![];
        if let Some(ep_authorization_type) = ep_authorization_type {
            endpoint_properties.push(ep_authorization_type)
        }
        if let Some(ep_authorization) = ep_authorization {
            endpoint_properties.push(ep_authorization)
        }
        DataAddress {
            _type: "DataAddress".to_string(),
            endpoint_type: self.endpoint_type.to_string(),
            endpoint: self.endpoint.to_string(),
            endpoint_properties,
        }
    }
}

pub(crate) fn conform_dataplane_forward_url(config: Arc<TransferConfig>, url: String) -> String {
    let base = config.common().get_host(HostType::Http);
    format!("{}{}", base, url)
}
