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

use common::dsp_common::data_address::{DataAddress, EndpointProperty};
use serde::{Deserialize, Serialize};

/// Internal DTO for RPC bodies and the data-plane facades. The wire form is
/// `common::dsp_common::data_address::DataAddress`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataAddressDto {
    pub endpoint_type: String,
    pub endpoint: Option<String>,
    pub endpoint_properties: Option<Vec<EndpointPropertyDto>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointPropertyDto {
    pub name: String,
    pub value: String,
}

/// Widen to the wire `DataAddress`. The `@type` tags are not in the DTO, so they
/// are defaulted: only the endpoint and its properties matter downstream.
impl From<DataAddressDto> for DataAddress {
    fn from(dto: DataAddressDto) -> Self {
        Self {
            _type: "DataAddress".to_string(),
            endpoint_type: dto.endpoint_type,
            endpoint: dto.endpoint,
            endpoint_properties: dto
                .endpoint_properties
                .unwrap_or_default()
                .into_iter()
                .map(|p| EndpointProperty {
                    _type: "EndpointProperty".to_string(),
                    name: p.name,
                    value: p.value,
                })
                .collect(),
        }
    }
}
