/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

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

pub(crate) mod transfer_messages;
pub(crate) mod transfer_process;
