/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| "out".to_string()));
    let descriptor_path = out_dir.join("transfer_ref_descriptor.bin");
    tonic_prost_build::configure()
        .compile_well_known_types(false)
        .file_descriptor_set_path(descriptor_path)
        .compile_protos(
            &[
                "proto/transfer_process.proto",
                "proto/transfer_messages.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
