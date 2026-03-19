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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use rainbow_common::config::types::traits::{EntityClientTrait, GaiaConfigTrait};
use ymir::config::traits::HostsConfigTrait;
use ymir::types::vcs::W3cDataModelVersion;

pub trait GaiaGaiaSelfIssuerConfigTrait:
    HostsConfigTrait + GaiaConfigTrait + EntityClientTrait
{
    fn is_local(&self) -> bool;
    fn get_data_model_version(&self) -> &W3cDataModelVersion;
    fn get_did(&self) -> &str;
}
