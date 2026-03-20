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

use crate::protocols::dsp::errors::error_adapter::DspNegotiationError;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use ymir::errors::{BadFormat, Errors};
use tracing::error;

pub(crate) mod error_adapter;

pub(crate) fn extract_payload_error<T>(
    input: Result<Json<T>, JsonRejection>,
) -> Result<T, DspNegotiationError> {
    match input {
        Ok(Json(data)) => Ok(data),
        Err(err) => {
            let e = Errors::format(BadFormat::Received, format!("{}", err.body_text()), None);
            error!("{}", e);
            Err(e.into())
        }
    }
}
