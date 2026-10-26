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

//! The `CatalogError` a peer receives (DSP 6.2.3).

use common::dsp_common::context_field::ContextField;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CatalogErrorDto {
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub kind: &'static str,
    pub code: String,
    pub reason: Vec<String>,
}

impl CatalogErrorDto {
    pub fn new(code: impl Into<String>, reason: Vec<String>) -> Self {
        Self {
            context: ContextField::default(),
            kind: "CatalogError",
            code: code.into(),
            reason,
        }
    }
}
