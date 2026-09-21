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

//! Agreement output views.

use crate::data::entities::agreement as agreement_model;
use crate::entities::agreement::AgreementDto;
use serde::{Deserialize, Serialize};

/// Management view of an agreement.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgreementView {
    #[serde(flatten)]
    pub inner: agreement_model::Model,
}

impl AgreementView {
    /// Assemble view from agreement model.
    pub fn assemble(inner: agreement_model::Model) -> Self {
        Self { inner }
    }
}

impl From<AgreementView> for AgreementDto {
    fn from(view: AgreementView) -> Self {
        Self { inner: view.inner }
    }
}

impl From<AgreementDto> for AgreementView {
    fn from(dto: AgreementDto) -> Self {
        Self { inner: dto.inner }
    }
}
