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

//! Requests and answers of the Gaia-X compliance actions.

use serde::{Deserialize, Serialize};

/// A Gaia-X compliance service to POST the presentation to, and the id of the credential to issue.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComplianceForm {
    pub url: String,
    #[serde(default)]
    pub vcid: Option<String>,
}

/// What came of presenting to a Gaia-X Dispatcher: the credential, or where it sends the holder.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DispatcherOutcome {
    pub received: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}
