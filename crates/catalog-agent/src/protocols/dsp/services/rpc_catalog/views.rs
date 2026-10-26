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

//! What the catalog RPC answers, as a eunomia `RpcCatalogResult`.

use common::eunomia::EUNOMIA_CONTEXT_URL;
use serde::Serialize;
use serde_json::Value;

/// What was asked and what the peer said, both as JSON literals.
#[derive(Debug, Clone, Serialize)]
pub struct RpcCatalogResult {
    #[serde(rename = "@context")]
    pub context: &'static str,
    #[serde(rename = "@type")]
    pub kind: &'static str,
    pub request: Value,
    pub response: Value,
}

impl RpcCatalogResult {
    pub const TYPE: &'static str = "RpcCatalogResult";

    pub fn assemble(request: Value, response: Value) -> Self {
        Self {
            context: EUNOMIA_CONTEXT_URL,
            kind: Self::TYPE,
            request,
            response,
        }
    }
}
