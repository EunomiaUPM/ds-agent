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

//! The DSP catalog message types (DSP 6.2).

use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ymir::errors::{BadFormat, Errors};

/// DSP catalog messages this agent sends and receives.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogDspMessageType {
    CatalogRequestMessage,
    DatasetRequestMessage,
    Catalog,
    Dataset,
    CatalogError,
}

impl CatalogDspMessageType {
    const ALL: [Self; 5] = [
        Self::CatalogRequestMessage,
        Self::DatasetRequestMessage,
        Self::Catalog,
        Self::Dataset,
        Self::CatalogError,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CatalogRequestMessage => "CatalogRequestMessage",
            Self::DatasetRequestMessage => "DatasetRequestMessage",
            Self::Catalog => "Catalog",
            Self::Dataset => "Dataset",
            Self::CatalogError => "CatalogError",
        }
    }
}

impl Display for CatalogDspMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CatalogDspMessageType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|t| t.as_str() == s)
            .ok_or_else(|| Errors::format(BadFormat::Received, format!("unknown catalog message {s}"), None))
    }
}
