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

use crate::config::oauth::OauthConfig;
use serde::{Deserialize, Serialize};
use ymir::config::traits::{
    ApiConfigTrait, ConnectionConfigTrait, DatabaseConfigTrait, HostsConfigTrait,
};
use ymir::config::types::{ApiConfig, CommonHostsConfig, ConnectionConfig, DatabaseConfig};

/// Hosts, database, API, connection flags and auth settings every agent section embeds.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommonConfig {
    hosts: CommonHostsConfig,
    db: DatabaseConfig,
    api: ApiConfig,
    connection: ConnectionConfig,
    #[serde(default)]
    pub oauth: OauthConfig,
}

impl HostsConfigTrait for CommonConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
}

impl DatabaseConfigTrait for CommonConfig {
    fn db(&self) -> &DatabaseConfig {
        &self.db
    }
}

impl ConnectionConfigTrait for CommonConfig {
    fn connection(&self) -> &ConnectionConfig {
        &self.connection
    }
}

impl ApiConfigTrait for CommonConfig {
    fn api(&self) -> &ApiConfig {
        &self.api
    }
}
