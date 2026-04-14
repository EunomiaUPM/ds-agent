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

use crate::entities::parameters::{TemplateMapString, TemplateString};
use crate::TemplateVecString;
use serde::{Deserialize, Serialize};

/// HTTP protocol specification.
///
/// All string fields support `{{__PARAM__}}` placeholders.  The `headers` map
/// values and `body_template` body string are each resolved individually.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpSpec {
    /// URL, possibly containing placeholders (e.g. `https://api.example.com/{{__ID__}}`).
    pub url_template: TemplateString,
    /// HTTP method(s).  Typically a single-element list such as `["GET"]`.
    pub method: TemplateVecString,
    /// Optional request headers map.  Values may contain placeholders.
    pub headers: Option<TemplateMapString>,
    /// Optional request body.  May be a JSON template string.
    pub body_template: Option<TemplateString>,
}
