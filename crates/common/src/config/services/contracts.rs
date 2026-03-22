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
use ymir::config::traits::ConnectionConfigTrait;
use ymir::config::types::ConnectionConfig;
use ymir::errors::Outcome;

use crate::config::services::traits::ContractsConfigTrait;
use crate::config::services::CommonConfig;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{CommonConfigTrait, ConfigLoader};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractsConfig {
    common: CommonConfig,
    ssi_auth: MinKnownConfig,
    is_catalog_datahub: bool,
}

impl ContractsConfigTrait for ContractsConfig {
    fn ssi_auth(&self) -> &MinKnownConfig {
        &self.ssi_auth
    }
    fn is_catalog_datahub(&self) -> bool {
        self.is_catalog_datahub
    }
}

impl ConfigLoader for ContractsConfig {
    fn load(env_file: &str) -> Outcome<Self> {
        Self::global_load(env_file)
            .map(|data| data.contracts().clone())
            .or_else(|_| Self::local_load(env_file))
    }
}

impl CommonConfigTrait for ContractsConfig {
    fn common(&self) -> &CommonConfig {
        &self.common
    }
}

impl ConnectionConfigTrait for ContractsConfig {
    fn connection(&self) -> &ConnectionConfig {
        self.common.connection()
    }
}
