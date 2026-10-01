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

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{CommonConfigTrait, ConfigLoader};

/// What the negotiation agent reads from its config.
pub trait ContractsConfigTrait: ConfigLoader + CommonConfigTrait {
    /// Address of the auth agent.
    fn ssi_auth(&self) -> &MinKnownConfig;
    /// Address of the catalog agent.
    fn catalog(&self) -> &MinKnownConfig;
    /// Whether the catalog is backed by a datahub.
    fn is_catalog_datahub(&self) -> bool;
}
