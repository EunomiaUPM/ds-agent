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

//! What the template API answers besides templates: usages and previews.

use serde::Serialize;
use serde_json::Value;

use crate::entities::policy_templates::ControlPoint;

/// Who uses a template version: other versions (`id@version`) and offers (their ids).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateUsagesView {
    pub templates: Vec<String>,
    pub offers: Vec<String>,
}

/// A composition flattened without storing it, and where its pieces are meant to apply.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyPreviewView {
    pub odrl: Value,
    pub control_points: Vec<ControlPoint>,
}
