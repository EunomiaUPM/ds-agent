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

//! A dataset published in one step together with its distribution and, optionally, its policy.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::datasets::DatasetDto;
use crate::entities::distributions::DistributionDto;
use crate::entities::odrl_policies::OdrlPolicyDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewDatasetOfferingDto {
    pub dataset: DatasetOfferingInput,
    pub distribution: DistributionOfferingInput,
    pub policy: Option<PolicyOfferingInput>,
}

/// Without `catalog_id` the dataset goes to the tenant's main catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatasetOfferingInput {
    pub title: String,
    pub description: Option<String>,
    pub conforms_to: Option<String>,
    pub creator: Option<String>,
    pub catalog_id: Option<String>,
}

/// Without `access_service_id` the distribution is served by the tenant's main data service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DistributionOfferingInput {
    pub title: String,
    pub description: Option<String>,
    pub formats: Option<String>,
    pub access_service_id: Option<String>,
}

/// A single-permission ODRL offer on the dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyOfferingInput {
    pub description: Option<String>,
    pub action: Option<String>,
    pub profile: Option<String>,
    pub constraints: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetOfferingDto {
    pub dataset: DatasetDto,
    pub distribution: DistributionDto,
    pub policy: Option<OdrlPolicyDto>,
}
