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

use serde::{Deserialize, Serialize};
use ymir::types::vcs::VcType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GaiaConfig {
    pub legal_person: LegalPersonInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LegalPersonInfo {
    pub registration_number: RegNumberInfo,
    pub legal_address: AddressInfo,
    pub headquarters_address: AddressInfo,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegNumberInfo {
    pub kind: VcType,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdivision_country_code: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressInfo {
    pub country_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street_address: Option<String>,
}
