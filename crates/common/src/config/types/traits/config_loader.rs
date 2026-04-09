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

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use ymir::errors::Outcome;
use ymir::utils::read;

use crate::config::ApplicationConfig;
use crate::utils::parse_yaml;

pub trait ConfigLoader: Sized + DeserializeOwned {
    fn load(env_file: &str) -> Outcome<Self>;
    fn global_load(env_file: &str) -> Outcome<ApplicationConfig> {
        ApplicationConfig::load(env_file)
    }

    fn local_load(env_file: &str) -> Outcome<Self> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(env_file);
        let data = read(path)?;
        parse_yaml(&data)
    }
}
