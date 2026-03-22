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

#![allow(unused)]
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::errors::helpers::BadFormat;
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use tracing::error;
use urn::Urn;

pub(crate) fn extract_payload<T>(input: Result<Json<T>, JsonRejection>) -> Result<T, Response> {
    match input {
        Ok(Json(data)) => Ok(data),
        Err(err) => {
            let e = CommonErrors::format_new(BadFormat::Received, &format!("{}", err.body_text()));
            error!("{}", e.log());
            Err(e.into_response())
        }
    }
}

pub(crate) fn parse_urn(id: &str) -> Result<Urn, Response> {
    Urn::from_str(id).map_err(|err| {
        let e = CommonErrors::format_new(
            BadFormat::Received,
            &format!("Urn malformed: {}. Error: {}", id, err),
        );
        error!("{}", e.log());
        e.into_response()
    })
}
