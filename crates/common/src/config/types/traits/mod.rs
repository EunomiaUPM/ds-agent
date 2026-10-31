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

//! Accessor traits shared by the agent configs.

mod cache_trait;
mod config_loader;
mod datahub;
mod entity_client;
mod gaia;
mod min_known;
mod oid4vp_service;

pub use cache_trait::CacheConfigTrait;
pub use config_loader::GlobalConfigLoader;
pub use datahub::DatahubConfigTrait;
pub use entity_client::EntityClientTrait;
pub use gaia::GaiaConfigTrait;
pub use min_known::MinKnownConfigTrait;
pub use oid4vp_service::Oid4VpServiceTrait;
pub use ymir::config::traits::{CommonConfigTrait, ConfigLoader};
