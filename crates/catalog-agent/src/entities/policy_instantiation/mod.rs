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

//! Filling one template with values to make an offer, and the checks each value goes through.

pub mod validators;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use urn::Urn;

use crate::entities::odrl_offer::CatalogEntityTypes;
use crate::entities::policy_templates::ParameterValue;

/// An offer made from one template version and the values of its parameters.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyInstantiationDto {
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,
    pub entity_id: Urn,
    pub entity_type: CatalogEntityTypes,
    pub description: Option<String>,
}
