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

//! gRPC driving adapter: generated API plus one handler module per resource.

pub(crate) mod transfer_messages;
pub(crate) mod transfer_process;

/// Generated protobuf/tonic code and the reflection descriptor set.
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
