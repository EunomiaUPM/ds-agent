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

pub mod config_builder;
pub(crate) mod dataplane_manager;
pub(crate) mod driver_factory;
pub use config_builder::{EgressConfig, IngressConfig};
pub(crate) mod dataplane_commands;
pub(crate) mod dataplane_persistence;

use common::config::types::roles::RoleConfig;
use common::dsp_common::data_address::{DataAddress, EndpointProperty};
use serde::{Deserialize, Serialize};
use urn::Urn;

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

#[derive(Debug)]
pub enum DataplaneCommand {
    /// Initiates dataplane. when transfer agent receives signals for creating a new TransferProcess
    /// Dataplane must be initiated
    /// Based on Role (defined when creating TransferProcess in transfer-agent
    SetInit(DataplaneInitCommandType),
    SetConfiguring,
    SetAuth,
    SetReady,
    SetStarted,
    SetSubscribing,
    SetUnsubscribing,
    SetStopped,
    SetTerminated,
    SetEgress {
        data_address: DataplaneAddress,
    },
}

#[derive(Debug)]
pub enum DataplaneInitCommandType {
    // If Provider, must know ConnectorInstance
    Provider {
        connector_instance: Urn,
        // If PUSH Case DataAddress must be provided by any means (DSP or others are consistent here)
        data_address: Option<DataplaneAddress>,
    },
    // If Consumer mustn't
    Consumer {
        // If PUSH Case DataAddress must be provided by any means (DSP or others are consistent here)
        data_address: Option<DataplaneAddress>,
    },
}

#[derive(Debug)]
pub enum DataplaneResponse {
    Ok,
    OkWithDataAddress(DataplaneAddress),
    Error(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug)]
pub struct DataplaneManagerInput {
    pub transfer_process_id: Urn,
    pub command: DataplaneCommand,
}
