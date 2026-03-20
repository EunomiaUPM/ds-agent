/*
 *
 *  * Copyright (C) 2024 - Universidad Politécnica de Madrid - UPM
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

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::io::Error;
use tracing::error;

pub trait CustomToResponse {
    fn to_response(&self) -> Response;
}

impl CustomToResponse for Error {
    fn to_response(&self) -> Response {
        error!("Unhandled internal error: {:?}", self);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Internal Server Error",
                "error_code": 5000
            })),
        )
            .into_response()
    }
}
