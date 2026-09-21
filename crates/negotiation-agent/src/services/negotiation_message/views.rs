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

//! Negotiation message output views.

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::negotiation_message as negotiation_message_model;
use crate::data::entities::offer as offer_model;
use crate::entities::negotiation_message::NegotiationMessageDto;
use serde::{Deserialize, Serialize};

/// Management view of a negotiation message with linked offer and agreement.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationMessageView {
    #[serde(flatten)]
    pub inner: negotiation_message_model::Model,
    pub offer: Option<offer_model::Model>,
    pub agreement: Option<agreement_model::Model>,
}

impl NegotiationMessageView {
    /// Assemble view from message model and optional linked offer and agreement.
    pub fn assemble(
        inner: negotiation_message_model::Model,
        offer: Option<offer_model::Model>,
        agreement: Option<agreement_model::Model>,
    ) -> Self {
        Self {
            inner,
            offer,
            agreement,
        }
    }
}

impl From<NegotiationMessageView> for NegotiationMessageDto {
    fn from(view: NegotiationMessageView) -> Self {
        Self {
            inner: view.inner,
            offer: view.offer,
            agreement: view.agreement,
        }
    }
}

impl From<NegotiationMessageDto> for NegotiationMessageView {
    fn from(dto: NegotiationMessageDto) -> Self {
        Self {
            inner: dto.inner,
            offer: dto.offer,
            agreement: dto.agreement,
        }
    }
}
