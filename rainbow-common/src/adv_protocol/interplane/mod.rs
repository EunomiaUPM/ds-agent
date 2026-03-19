/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ymir::errors::Errors;
use crate::dcat_formats::FormatAction;

pub mod data_plane_provision;
pub mod data_plane_start;
pub mod data_plane_status;
pub mod data_plane_stop;

#[derive(Serialize, Deserialize, Debug)]
pub enum DataPlaneControllerVersion {
    #[serde(rename = "1.0")]
    Version10
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataPlaneControllerMessages {
    #[serde(rename = "DataPlaneProvisionRequest")]
    DataPlaneProvisionRequest,
    #[serde(rename = "DataPlaneProvisionResponse")]
    DataPlaneProvisionResponse,
    #[serde(rename = "DataPlaneStatusRequest")]
    DataPlaneStatusRequest,
    #[serde(rename = "DataPlaneStatusResponse")]
    DataPlaneStatusResponse,
    #[serde(rename = "DataPlaneStart")]
    DataPlaneStart,
    #[serde(rename = "DataPlaneStartAck")]
    DataPlaneStartAck,
    #[serde(rename = "DataPlaneStop")]
    DataPlaneStop,
    #[serde(rename = "DataPlaneStopAck")]
    DataPlaneStopAck
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum DataPlaneSDPFieldTypes {
    #[serde(rename = "DataPlaneAddressScheme")]
    DataPlaneAddressScheme,
    #[serde(rename = "DataPlaneAddress")]
    DataPlaneAddress,
    #[serde(rename = "DataPlaneAddressAuthType")]
    DataPlaneAddressAuthType,
    #[serde(rename = "DataPlaneAddressAuthToke")]
    DataPlaneAddressAuthToken
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum DataPlaneSDPConfigTypes {
    #[serde(rename = "NextHopAddressScheme")]
    NextHopAddressScheme,
    #[serde(rename = "NextHopAddress")]
    NextHopAddress,
    #[serde(rename = "NextHopConfigurationBody")]
    NextHopConfigurationBody,
    #[serde(rename = "NextHopAddressAuth")]
    NextHopAddressAuth,
    #[serde(rename = "NextHopAddressAuthType")]
    NextHopAddressAuthType,
    #[serde(rename = "Direction")]
    Direction
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPlaneSDPRequestField {
    #[serde(rename = "@type")]
    pub _type: DataPlaneSDPFieldTypes,
    pub format: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DataPlaneSDPResponseField {
    #[serde(rename = "@type")]
    pub _type: DataPlaneSDPFieldTypes,
    pub format: String,
    pub content: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DataPlaneSDPConfigField {
    #[serde(rename = "@type")]
    pub _type: DataPlaneSDPConfigTypes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub content: String
}

#[derive(Debug, Serialize)]
pub enum DataPlaneProcessDirection {
    PUSH,
    PULL,
    BIDI
}

impl FromStr for DataPlaneProcessDirection {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PUSH" => Ok(DataPlaneProcessDirection::PUSH),
            "PULL" => Ok(DataPlaneProcessDirection::PULL),
            "BIDI" => Ok(DataPlaneProcessDirection::BIDI),
            _ => Err(Errors::crazy("no direction allowed", None))
        }
    }
}

impl Display for DataPlaneProcessDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataPlaneProcessDirection::PUSH => f.write_str("PUSH"),
            DataPlaneProcessDirection::PULL => f.write_str("PULL"),
            DataPlaneProcessDirection::BIDI => f.write_str("BIDI")
        }
    }
}

impl From<FormatAction> for DataPlaneProcessDirection {
    fn from(value: FormatAction) -> Self {
        match value {
            FormatAction::Push => DataPlaneProcessDirection::PUSH,
            FormatAction::Pull => DataPlaneProcessDirection::PULL
        }
    }
}

#[derive(Debug, Serialize)]
pub enum DataPlaneProcessState {
    REQUESTED,
    STARTED,
    STOPPED,
    TERMINATED
}

impl FromStr for DataPlaneProcessState {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REQUESTED" => Ok(DataPlaneProcessState::REQUESTED),
            "STARTED" => Ok(DataPlaneProcessState::STARTED),
            "STOPPED" => Ok(DataPlaneProcessState::STOPPED),
            "TERMINATED" => Ok(DataPlaneProcessState::TERMINATED),
            _ => Err(Errors::crazy("no state allowed", None))
        }
    }
}

impl Display for DataPlaneProcessState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataPlaneProcessState::REQUESTED => f.write_str("REQUESTED"),
            DataPlaneProcessState::STARTED => f.write_str("STARTED"),
            DataPlaneProcessState::STOPPED => f.write_str("STOPPED"),
            DataPlaneProcessState::TERMINATED => f.write_str("TERMINATED")
        }
    }
}
