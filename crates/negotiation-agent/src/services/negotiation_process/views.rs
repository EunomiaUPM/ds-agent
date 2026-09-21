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

//! Negotiation process output views.

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::negotiation_message as negotiation_message_model;
use crate::data::entities::negotiation_process as negotiation_process_model;
use crate::data::entities::offer as offer_model;
use crate::entities::negotiation_process::NegotiationProcessDto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Management view of a negotiation process with its identifiers and linked entities.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationProcessView {
    #[serde(flatten)]
    pub inner: negotiation_process_model::Model,
    pub identifiers: HashMap<String, String>,
    pub messages: Vec<negotiation_message_model::Model>,
    pub offers: Vec<offer_model::Model>,
    pub agreement: Option<agreement_model::Model>,
}

impl NegotiationProcessView {
    /// Assemble view from model and related sub-entities.
    pub fn assemble(
        inner: negotiation_process_model::Model,
        identifiers: HashMap<String, String>,
        messages: Vec<negotiation_message_model::Model>,
        offers: Vec<offer_model::Model>,
        agreement: Option<agreement_model::Model>,
    ) -> Self {
        Self {
            inner,
            identifiers,
            messages,
            offers,
            agreement,
        }
    }
}

impl From<NegotiationProcessView> for NegotiationProcessDto {
    fn from(view: NegotiationProcessView) -> Self {
        Self {
            inner: view.inner,
            identifiers: view.identifiers,
            messages: view.messages,
            offers: view.offers,
            agreement: view.agreement,
        }
    }
}

impl From<NegotiationProcessDto> for NegotiationProcessView {
    fn from(dto: NegotiationProcessDto) -> Self {
        Self {
            inner: dto.inner,
            identifiers: dto.identifiers,
            messages: dto.messages,
            offers: dto.offers,
            agreement: dto.agreement,
        }
    }
}
