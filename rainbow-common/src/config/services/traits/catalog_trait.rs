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

use crate::config::types::traits::{
    CacheConfigTrait, CommonConfigTrait, ConfigLoader, DatahubConfigTrait,
};
use crate::config::types::cache::CacheConfig;
use crate::config::types::min_known_config::MinKnownConfig;

pub trait CatalogConfigTrait:
    ConfigLoader + CommonConfigTrait + DatahubConfigTrait + CacheConfigTrait
{
    fn contracts(&self) -> &MinKnownConfig;

    fn ssi_auth(&self) -> &MinKnownConfig;
    fn cache(&self) -> &CacheConfig;
    fn is_datahub(&self) -> bool;

    fn get_policy_templates_folder(&self) -> &str;
}
