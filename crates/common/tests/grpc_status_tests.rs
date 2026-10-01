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

use common::errors::ResourceError;
use common::grpc::IntoStatus;
use tonic::Code;
use ymir::errors::{BadFormat, Errors};

#[test]
fn not_found_maps_to_not_found_with_reason() {
    let status = ResourceError::not_found("urn:x:1", "dataset").into_status();
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(status.message(), "dataset not found");
}

#[test]
fn forbidden_maps_to_permission_denied() {
    assert_eq!(
        Errors::forbidden("nope", None).into_status().code(),
        Code::PermissionDenied
    );
}

#[test]
fn unauthorized_maps_to_unauthenticated() {
    assert_eq!(
        Errors::unauthorized("nope", None).into_status().code(),
        Code::Unauthenticated
    );
}

#[test]
fn bad_request_format_maps_to_invalid_argument() {
    assert_eq!(
        Errors::format(BadFormat::Received, "bad", None)
            .into_status()
            .code(),
        Code::InvalidArgument
    );
}

#[test]
fn upstream_format_maps_to_unavailable() {
    assert_eq!(
        Errors::format(BadFormat::Sent, "bad", None)
            .into_status()
            .code(),
        Code::Unavailable
    );
}

#[test]
fn not_implemented_maps_to_unimplemented() {
    assert_eq!(
        Errors::not_impl("later", None).into_status().code(),
        Code::Unimplemented
    );
}

#[test]
fn internal_errors_map_to_internal() {
    assert_eq!(
        Errors::crazy("boom", None).into_status().code(),
        Code::Internal
    );
}
