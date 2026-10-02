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

use crate::config::types::cache::CacheConfig;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{
    CacheConfigTrait, CommonConfigTrait, ConfigLoader, DatahubConfigTrait,
};

/// What the catalog agent reads from its config.
pub trait CatalogConfigTrait:
    ConfigLoader + CommonConfigTrait + DatahubConfigTrait + CacheConfigTrait
{
    /// Address of the negotiation agent.
    fn contracts(&self) -> &MinKnownConfig;

    /// Address of the auth agent.
    fn ssi_auth(&self) -> &MinKnownConfig;
    fn cache(&self) -> &CacheConfig;
    /// Whether the catalog is backed by a datahub instead of the database.
    fn is_datahub(&self) -> bool;

    /// Folder of policy templates loaded at boot.
    fn get_policy_templates_folder(&self) -> &str;
}
