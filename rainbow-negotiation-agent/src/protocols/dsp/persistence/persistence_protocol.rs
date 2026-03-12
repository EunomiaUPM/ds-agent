#![allow(unused)]
/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

// NOTE: This file is retained for potential future use.
// The active inbound-protocol persistence is `OrchestrationPersistenceForProtocol`
// in `orchestrator/protocol/persistence.rs`.

use crate::entities::agreement::NegotiationAgentAgreementsTrait;
use crate::entities::negotiation_message::NegotiationAgentMessagesTrait;
use crate::entities::negotiation_process::NegotiationAgentProcessesTrait;
use crate::entities::offer::NegotiationAgentOffersTrait;
use std::sync::Arc;

pub struct NegotiationPersistenceForProtocolService {
    pub negotiation_process_service: Arc<dyn NegotiationAgentProcessesTrait>,
    pub negotiation_messages_service: Arc<dyn NegotiationAgentMessagesTrait>,
    pub offer_service: Arc<dyn NegotiationAgentOffersTrait>,
    pub agreement_service: Arc<dyn NegotiationAgentAgreementsTrait>,
}

impl NegotiationPersistenceForProtocolService {
    pub fn new(
        negotiation_process_service: Arc<dyn NegotiationAgentProcessesTrait>,
        negotiation_messages_service: Arc<dyn NegotiationAgentMessagesTrait>,
        offer_service: Arc<dyn NegotiationAgentOffersTrait>,
        agreement_service: Arc<dyn NegotiationAgentAgreementsTrait>,
    ) -> Self {
        Self {
            negotiation_process_service,
            negotiation_messages_service,
            offer_service,
            agreement_service,
        }
    }
}
