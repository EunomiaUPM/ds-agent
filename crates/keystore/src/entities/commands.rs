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

use crate::entities::key::Key;
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewParameterCommand<T> {
    pub key: Key,
    pub value: T,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditParameterCommand<T> {
    pub value: T,
    pub expected_version: Version,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewSecretCommand {
    pub key: Key,
    pub value: SecretValue,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSecretCommand {
    pub value: SecretValue,
    pub expected_version: Version,
    pub description: Option<String>,
}
