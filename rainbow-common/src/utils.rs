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

use axum::response::{IntoResponse, Response};
use std::str::FromStr;
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
    string_in
        .parse::<Urn>()
        .map_err(|e| Errors::parse("Error parsing urn", Some(anyhow::Error::from(e))))
}

pub fn parse_urn(id: &str) -> Result<Urn, Response> {
    Urn::from_str(id).map_err(|err| {
        let e = Errors::parse("Error parsing urn", Some(anyhow::Error::from(err)));
        e.log();
        e.into_response()
    })
}

pub async fn flush_redis_cache(url: &str) -> Outcome<()> {
    tracing::info!("Connecting to Redis at {}...", url);
    // CREAR NUEVO ERROR DE REDSIS?
    let client = redis::Client::open(url).map_err(|err| todo!())?;
    let mut con = client.get_async_connection().await.map_err(|err| todo!())?;
    redis::cmd("FLUSHALL").query_async::<_, ()>(&mut con).await.map_err(|err| todo!())?;
    tracing::info!("Redis cache flushed successfully.");
    Ok(())
}
