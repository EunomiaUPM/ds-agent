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

use axum::http::StatusCode;
use tonic::Status;
use ymir::errors::Errors;

pub(crate) mod transfer_messages;
pub(crate) mod transfer_process;
mod utils;

// Extracts public gRPC API from build stage
pub mod api {
    pub mod transfer_processes {
        tonic::include_proto!("transfer_processes_ref");
    }
    pub mod transfer_messages {
        tonic::include_proto!("transfer_messages_ref");
    }
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("transfer_ref_descriptor");
}

/// Translates a domain [`Errors`] into a gRPC [`Status`]
pub(crate) fn to_status(err: Errors) -> Status {
    let message = err.reason().to_string();
    match err.info().status_code {
        StatusCode::NOT_FOUND => Status::not_found(message),
        StatusCode::FORBIDDEN => Status::permission_denied(message),
        StatusCode::UNAUTHORIZED => Status::unauthenticated(message),
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
            Status::invalid_argument(message)
        }
        StatusCode::PRECONDITION_FAILED => Status::failed_precondition(message),
        _ => Status::internal(message),
    }
}
