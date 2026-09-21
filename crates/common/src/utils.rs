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

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::str::FromStr;
use tracing::info;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::{Errors, Outcome};

static UUID_PREFIX: &str = "urn:uuid:";

pub fn get_urn(optional_urn: Option<Urn>) -> Urn {
    optional_urn.unwrap_or_else(|| {
        let uuid = Uuid::new_v4();
        let id_string = format!("{}{}", UUID_PREFIX, uuid);
        let urn = id_string.parse::<Urn>().unwrap();
        urn
    })
}

pub fn generate_uuid_urn(prefix: &str) -> Urn {
    Urn::from_str(&format!("urn:{}:{}", prefix, uuid::Uuid::new_v4()))
        .expect("UUID URN is always valid")
}

/// Parses a string slice into a `Urn`.
#[allow(clippy::result_large_err)]
pub fn parse_urn(s: &str) -> Outcome<Urn> {
    s.parse::<Urn>()
        .map_err(|e| Errors::crazy("invalid URN in database", Some(Box::new(e))))
}

/// Parses a string slice into a `Urn` (backwards-compatible alias).
#[allow(clippy::result_large_err)]
pub fn get_urn_from_string(string_in: &str) -> Outcome<Urn> {
    parse_urn(string_in)
}

/// Extension trait for parsing string slices into URNs.
pub trait ParseUrnExt {
    /// Parses the string slice into a `Urn`.
    #[allow(clippy::result_large_err)]
    fn parse_urn(&self) -> Outcome<Urn>;
}

impl ParseUrnExt for str {
    #[allow(clippy::result_large_err)]
    fn parse_urn(&self) -> Outcome<Urn> {
        parse_urn(self)
    }
}

pub async fn flush_redis_cache(url: &str) -> Outcome<()> {
    info!("Connecting to Redis at {}...", url);
    // NEW REDS IS ERROR?
    let client = redis::Client::open(url)
        .map_err(|err| Errors::crazy("Redis client open url failed", Some(Box::new(err))))?;
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| {
            Errors::crazy("Redis getting async connection failed", Some(Box::new(err)))
        })?;
    redis::cmd("FLUSHALL")
        .query_async::<()>(&mut con)
        .await
        .map_err(|err| Errors::crazy("Redis command failed", Some(Box::new(err))))?;
    info!("Redis cache flushed successfully.");
    Ok(())
}

pub fn show_table(config: &impl Serialize) -> Outcome<()> {
    let table = json_to_table::json_to_table(
        &serde_json::to_value(config)
            .map_err(|e| Errors::parse("Error with config table", Some(Box::new(e))))?,
    )
    .collapse()
    .to_string();
    info!("Current Config:\n{}", table);
    Ok(())
}

pub fn parse_yaml<T: DeserializeOwned>(path: &str) -> Outcome<T> {
    serde_norway::from_str(path)
        .map_err(|e| Errors::parse("Unable to parse config file", Some(Box::new(e))))
}

pub fn json_merge(base: &mut serde_json::Value, patch: serde_json::Value) {
    if let (serde_json::Value::Object(base_map), serde_json::Value::Object(patch_map)) =
        (base, patch)
    {
        for (k, v) in patch_map {
            base_map.insert(k, v);
        }
    }
}
