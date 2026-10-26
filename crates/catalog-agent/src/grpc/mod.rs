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

//! gRPC driving adapter, the simplest mirror of the management API: resources travel as the
//! same JSON-LD documents, offers and templates as the same JSON.

pub mod odrl_offers;
pub mod policy_templates;
pub mod resources;

/// Generated protobuf/tonic code and the reflection descriptor set.
pub mod api {
    /// Generated `catalog.v1` messages and services.
    pub mod catalog_agent {
        tonic::include_proto!("catalog.v1");
    }

    /// Encoded descriptors for the reflection service.
    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("catalog_descriptor");
}

use serde::Serialize;
use tonic::Status;

use crate::grpc::api::catalog_agent::JsonDocument;

/// JSON answers of the gRPC mirror.
pub struct GrpcJson;

impl GrpcJson {
    pub fn document<T: Serialize>(value: &T) -> Result<JsonDocument, Status> {
        serde_json::to_string(value)
            .map(|json| JsonDocument { json })
            .map_err(|e| Status::internal(format!("answer is not serializable: {e}")))
    }

    /// The JSON `field` carries, as the type it must be.
    pub fn parse<T: serde::de::DeserializeOwned>(json: &str, field: &str) -> Result<T, Status> {
        serde_json::from_str(json).map_err(|e| Status::invalid_argument(format!("{field}: {e}")))
    }
}
