/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or("out".to_string()));
    let descriptor_path = out_dir.join("negotiation_descriptor.bin");
    tonic_prost_build::configure()
        .compile_well_known_types(false)
        .file_descriptor_set_path(descriptor_path)
        .compile_protos(
            &[
                "proto/models.proto",
                "proto/negotiation-process.proto",
                "proto/negotiation-message.proto",
                "proto/offer.proto",
                "proto/agreement.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
