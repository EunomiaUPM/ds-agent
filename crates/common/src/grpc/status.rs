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

//! Maps domain `Errors` onto gRPC `Status` codes.

use axum::http::StatusCode;
use tonic::Status;
use ymir::errors::Errors;

/// Conversion of a domain error into a gRPC `Status`.
pub trait IntoStatus {
    fn into_status(self) -> Status;
}

impl IntoStatus for Errors {
    fn into_status(self) -> Status {
        let message = self.reason().to_string();
        match self.info().status_code {
            StatusCode::NOT_FOUND => Status::not_found(message),
            StatusCode::FORBIDDEN => Status::permission_denied(message),
            StatusCode::UNAUTHORIZED => Status::unauthenticated(message),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                Status::invalid_argument(message)
            }
            StatusCode::CONFLICT => Status::already_exists(message),
            StatusCode::PRECONDITION_FAILED => Status::failed_precondition(message),
            StatusCode::TOO_MANY_REQUESTS => Status::resource_exhausted(message),
            StatusCode::NOT_IMPLEMENTED => Status::unimplemented(message),
            StatusCode::BAD_GATEWAY | StatusCode::SERVICE_UNAVAILABLE => {
                Status::unavailable(message)
            }
            _ => Status::internal(message),
        }
    }
}
