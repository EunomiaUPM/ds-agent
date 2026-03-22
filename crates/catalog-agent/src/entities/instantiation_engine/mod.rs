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

pub(crate) mod instantiation_engine;
pub(crate) mod validator_request;
pub(crate) mod validators;

use crate::entities::common::PolicyTemplateAllowedDefaultValues;
use crate::entities::odrl_policies::CatalogEntityTypes;
use crate::OdrlPolicyDto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyInstantiationDto {
    id: String,
    version: String,
    parameters: HashMap<String, PolicyTemplateAllowedDefaultValues>,
    entity_id: Urn,
    entity_type: CatalogEntityTypes,
    pub description: Option<String>,
}

#[async_trait::async_trait]
pub trait PolicyInstantiationTrait: Send + Sync {
    async fn instantiate_policy(
        &self,
        instantiation_request: &NewPolicyInstantiationDto,
    ) -> Outcome<OdrlPolicyDto>;
}
