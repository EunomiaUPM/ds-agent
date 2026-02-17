pub(crate) mod dataplane_manager;
pub(crate) mod driver_factory;
pub(crate) mod config_builder;
pub(crate) mod tests;

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use urn::Urn;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::dsp_common::data_address::DataAddress;

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
            authorization_type: self.endpoint_properties.iter()
                .find(|p| p.name == "authType")
                .map(|p| p.value.clone()),
            authorization: self.endpoint_properties.iter()
                .find(|p| p.name == "authorization")
                .map(|p| p.value.clone()),
        }
    }
}

pub enum DataplaneCommand {
    SetInit {
        role: RoleConfig,
        connector_instance: Urn, // Connector instance URN,
        data_address: Option<DataplaneAddress>,
    },
    SetConfiguring,
    SetAuth,
    SetReady,
    SetSubscribing,
    SetStarted,
    SetUnsubscribing,
    SetStopped,
    SetTerminated,
}

pub enum DataplaneResponse {
    Ok,
    OkWithDataAddress(DataplaneAddress),
    Error(Error),
}

pub struct DataplaneManagerInput {
    pub transfer_process_id: Urn,
    pub command: DataplaneCommand,
}

