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

use std::fmt::Display;
use std::ops::Not;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ymir::errors::Errors;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum RoleConfig {
    NotDefined,
    Consumer,
    Provider,
}

impl Not for RoleConfig {
    type Output = RoleConfig;

    fn not(self) -> Self::Output {
        match self {
            RoleConfig::NotDefined => self,
            RoleConfig::Consumer => RoleConfig::Provider,
            RoleConfig::Provider => RoleConfig::Consumer,
        }
    }
}

impl FromStr for RoleConfig {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Consumer" => Ok(RoleConfig::Consumer),
            "Provider" => Ok(RoleConfig::Provider),
            "" => Ok(RoleConfig::NotDefined),
            _ => Err(Errors::crazy(format!("Invalid config role: {}", s), None)),
        }
    }
}

impl Display for RoleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RoleConfig::Consumer => "Consumer".to_string(),
            RoleConfig::Provider => "Provider".to_string(),
            RoleConfig::NotDefined => "Not defined".to_string(),
        };
        write!(f, "{}", str)
    }
}
