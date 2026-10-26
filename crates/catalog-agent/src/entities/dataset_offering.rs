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

//! A dataset published in one step with its distribution and, optionally, a policy. Both
//! resources are management API documents; the policy is a short ODRL offer.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Dataset, distribution and optional policy published in one call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewDatasetOfferingCommand {
    /// A `Dataset`; without `inCatalog` it goes to the connector's main catalog.
    pub dataset: Value,
    /// A `Distribution` of it; without `accessService` the main data service serves it.
    pub distribution: Value,
    pub policy: Option<PolicyOfferingInput>,
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
