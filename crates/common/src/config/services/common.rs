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

use crate::config::types::AdminSeedConfig;
use serde::{Deserialize, Serialize};
use ymir::config::traits::{
    ApiConfigTrait, ConnectionConfigTrait, DatabaseConfigTrait, HostsConfigTrait,
};
use ymir::config::types::{ApiConfig, CommonHostsConfig, ConnectionConfig, DatabaseConfig};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommonConfig {
    pub hosts: CommonHostsConfig,
    pub db: DatabaseConfig,
    pub api: ApiConfig,
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub jwt_secret: String,
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl: i64,
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl: i64,
    #[serde(default)]
    pub admin_seed: AdminSeedConfig,
}

fn default_access_token_ttl() -> i64 {
    3_600
}

fn default_refresh_token_ttl() -> i64 {
    2_592_000
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
