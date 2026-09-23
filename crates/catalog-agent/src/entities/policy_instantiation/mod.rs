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

pub(crate) mod validator_request;
pub(crate) mod validators;

use crate::entities::odrl_policies::CatalogEntityTypes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyInstantiationDto {
    pub id: String,
    pub version: String,
    pub parameters: HashMap<String, PolicyTemplateAllowedDefaultValues>,
    pub entity_id: Urn,
    pub entity_type: CatalogEntityTypes,
    pub description: Option<String>,
}

use crate::entities::policy_templates::PolicyTemplateAllowedDefaultValues;
