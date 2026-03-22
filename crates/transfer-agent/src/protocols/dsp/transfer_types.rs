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

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ymir::errors::Errors;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferState {
    #[serde(rename = "REQUESTED")]
    REQUESTED,
    #[serde(rename = "STARTED")]
    STARTED,
    #[serde(rename = "TERMINATED")]
    TERMINATED,
    #[serde(rename = "COMPLETED")]
    COMPLETED,
    #[serde(rename = "SUSPENDED")]
    SUSPENDED,
}

impl FromStr for TransferState {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "REQUESTED" => Ok(Self::REQUESTED),
            "STARTED" => Ok(Self::STARTED),
            "TERMINATED" => Ok(Self::TERMINATED),
            "COMPLETED" => Ok(Self::COMPLETED),
            "SUSPENDED" => Ok(Self::SUSPENDED),
            _ => Err(Errors::parse("State not recognized", None)),
        }
    }
}

impl fmt::Display for TransferState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferState::REQUESTED => f.write_str("REQUESTED"),
            TransferState::STARTED => f.write_str("STARTED"),
            TransferState::TERMINATED => f.write_str("TERMINATED"),
            TransferState::COMPLETED => f.write_str("COMPLETED"),
            TransferState::SUSPENDED => f.write_str("SUSPENDED"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferStateAttribute {
    #[serde(rename = "ON_REQUEST")]
    OnRequest,
    #[serde(rename = "BY_PROVIDER")]
    ByProvider,
    #[serde(rename = "BY_CONSUMER")]
    ByConsumer,
}

impl FromStr for TransferStateAttribute {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ON_REQUEST" => Ok(Self::OnRequest),
            "BY_PROVIDER" => Ok(Self::ByProvider),
            "BY_CONSUMER" => Ok(Self::ByConsumer),
            _ => Err(Errors::parse("State Attribute not recognized", None)),
        }
    }
}

impl fmt::Display for TransferStateAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferStateAttribute::OnRequest => f.write_str("ON_REQUEST"),
            TransferStateAttribute::ByProvider => f.write_str("BY_PROVIDER"),
            TransferStateAttribute::ByConsumer => f.write_str("BY_CONSUMER"),
        }
    }
}
