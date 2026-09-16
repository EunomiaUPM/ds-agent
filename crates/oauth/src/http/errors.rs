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

use axum::http::{header, HeaderValue, StatusCode};
use axum::Json;
use axum::response::{IntoResponse, Response};
pub(crate) use crate::entities::errors::{OAuthError, OAuthErrorCode};

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let status = match self.error {
            OAuthErrorCode::InvalidClient => StatusCode::UNAUTHORIZED,
            OAuthErrorCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        };

        let mut resp = (status, Json(self.clone())).into_response();
        if self.error == OAuthErrorCode::InvalidClient {
            resp.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Basic realm=\"oauth\""),
            );
        }
        resp
    }
}
