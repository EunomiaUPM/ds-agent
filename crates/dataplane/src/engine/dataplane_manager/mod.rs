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
            // The wire `endpoint` is optional (DSP Appendix A); the data plane
            // cannot work without one. That it is present here is guaranteed by
            // the domain rule that requires it for the transfer kinds that reach
            // this conversion, not by the wire shape.
            endpoint: self.endpoint.unwrap_or_default(),
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
            endpoint: Some(self.endpoint.to_string()),
            endpoint_properties,
        }
    }
}

pub(crate) fn conform_dataplane_forward_url(config: Arc<TransferConfig>, url: String) -> String {
    let base = config.common().get_host(HostType::Http);
    format!("{}{}", base, url)
}
