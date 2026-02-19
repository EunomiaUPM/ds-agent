/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::errors::helpers::BadFormat;
use crate::errors::{CommonErrors, ErrorLog};
use anyhow::bail;
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::str::FromStr;
use tracing::error;
use urn::Urn;
use uuid::Uuid;
use ymir::config::types::HostConfig;

static UUID_PREFIX: &str = "urn:uuid:";

pub fn get_urn(optional_urn: Option<Urn>) -> Urn {
    optional_urn.unwrap_or_else(|| {
        let uuid = Uuid::new_v4();
        let id_string = format!("{}{}", UUID_PREFIX, uuid);
        let urn = id_string.parse::<Urn>().unwrap();
        urn
    })
}

pub fn get_urn_from_string(string_in: &String) -> anyhow::Result<Urn> {
    let urn_res = string_in.parse::<Urn>();
    match urn_res {
        Ok(urn_res) => Ok(urn_res),
        Err(e) => {
            let e_ = CommonErrors::format_new(BadFormat::Received, &e.to_string());
            error!("{}", e_.log());
            bail!(e_);
        }
    }
}

pub fn get_host_helper(host: Option<&HostConfig>, module: &str) -> anyhow::Result<String> {
    match host {
        Some(host) => match host.port.as_ref() {
            Some(port) => Ok(format!("{}://{}:{}", host.protocol, host.url, port)),
            None => Ok(format!("{}://{}", host.protocol, host.url)),
        },
        None => {
            let error = CommonErrors::module_new(module);
            error!("{}", error.log());
            bail!(error)
        }
    }
}

pub fn extract_payload<T>(input: Result<Json<T>, JsonRejection>) -> Result<T, Response> {
    match input {
        Ok(Json(data)) => Ok(data),
        Err(err) => {
            let e = CommonErrors::format_new(BadFormat::Received, &format!("{}", err.body_text()));
            error!("{}", e.log());
            Err(e.into_response())
        }
    }
}

pub fn parse_urn(id: &str) -> Result<Urn, Response> {
    Urn::from_str(id).map_err(|err| {
        let e = CommonErrors::format_new(
            BadFormat::Received,
            &format!("Urn malformed: {}. Error: {}", id, err),
        );
        error!("{}", e.log());
        e.into_response()
    })
}

pub async fn flush_redis_cache(url: &str) -> anyhow::Result<()> {
    tracing::info!("Connecting to Redis at {}...", url);
    let client = redis::Client::open(url)?;
    let mut con = client.get_async_connection().await?;
    redis::cmd("FLUSHALL").query_async::<_, ()>(&mut con).await?;
    tracing::info!("Redis cache flushed successfully.");
    Ok(())
}
