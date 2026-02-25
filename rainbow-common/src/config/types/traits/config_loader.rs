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

use crate::config::ApplicationConfig;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use ymir::errors::{Errors, Outcome};

pub trait ConfigLoader: Sized + DeserializeOwned {
    fn load(env_file: &str) -> Self;
    fn global_load(env_file: &str) -> Outcome<ApplicationConfig> {
        ApplicationConfig::load(env_file)
    }

    fn local_load(env_file: &str) -> Outcome<Self> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(env_file);
        let data = fs::read_to_string(&path).expect("Cannot read local config");
        serde_norway::from_str(&data)
            .map_err(|e| Errors::parse("Unable to parse config file", Some(anyhow::Error::from(e))))
    }
}
