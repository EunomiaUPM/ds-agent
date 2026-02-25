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

use crate::config::types::traits::CacheConfigTrait;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheConfig {
    pub cache_type: CacheType,
    pub url: String,
    pub port: String,
    pub user: String,
    pub password: String,
}

impl CacheConfigTrait for CacheConfig {
    fn cache_config(&self) -> &CacheConfig {
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CacheType {
    Redis,
    Memcached,
    Memory,
    Noop,
}

impl Default for CacheType {
    fn default() -> Self {
        Self::Noop
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: CacheType::Noop,
            url: "".to_string(),
            port: "".to_string(),
            user: "".to_string(),
            password: "".to_string(),
        }
    }
}

impl Display for CacheType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheType::Redis => write!(f, "redis"),
            CacheType::Memcached => write!(f, "memcached"),
            CacheType::Memory => write!(f, "memory"),
            CacheType::Noop => write!(f, "noop"),
        }
    }
}

impl FromStr for CacheType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "redis" => Ok(CacheType::Redis),
            "memcached" => Ok(CacheType::Memcached),
            "memory" => Ok(CacheType::Memory),
            "noop" => Ok(CacheType::Noop),
            _ => Err(anyhow!("error")),
        }
    }
}

impl FromStr for &CacheType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "redis" => Ok(&CacheType::Redis),
            "memcached" => Ok(&CacheType::Memcached),
            "memory" => Ok(&CacheType::Memory),
            "noop" => Ok(&CacheType::Noop),
            _ => Err(anyhow!("error")),
        }
    }
}
