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
use serde::de::DeserializeOwned;
use serde::Serialize;
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

pub fn get_urn_from_string(string_in: &String) -> Outcome<Urn> {
    string_in.parse::<Urn>().map_err(|e| Errors::parse("Error parsing urn", Some(Box::new(e))))
}

pub async fn flush_redis_cache(url: &str) -> Outcome<()> {
    info!("Connecting to Redis at {}...", url);
    // NEW REDS IS ERROR?
    let client = redis::Client::open(url)
        .map_err(|err| Errors::crazy("Redis client open url failed", Some(Box::new(err))))?;
    let mut con = client.get_multiplexed_async_connection().await.map_err(|err| {
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
            .map_err(|e| Errors::parse("Error with config table", Some(Box::new(e))))?
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
