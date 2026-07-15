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

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ymir::errors::Errors;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransferDSPState {
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

impl FromStr for TransferDSPState {
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

impl fmt::Display for TransferDSPState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferDSPState::REQUESTED => f.write_str("REQUESTED"),
            TransferDSPState::STARTED => f.write_str("STARTED"),
            TransferDSPState::TERMINATED => f.write_str("TERMINATED"),
            TransferDSPState::COMPLETED => f.write_str("COMPLETED"),
            TransferDSPState::SUSPENDED => f.write_str("SUSPENDED"),
        }
    }
}
