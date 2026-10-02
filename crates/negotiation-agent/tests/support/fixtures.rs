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

//! URNs and stored rows of the stub tenant. Auth doubles come from `common::test_utils::grpc`.

use chrono::Utc;
use common::test_utils::grpc::TENANT;
use negotiation_agent::data::entities::{agreement, negotiation_message, offer};
use serde_json::json;

pub fn urn(n: u32) -> String {
    format!("urn:uuid:00000000-0000-0000-0000-{n:012}")
}

pub fn message_row(n: u32) -> negotiation_message::Model {
    negotiation_message::Model {
        id: urn(n),
        tenant_id: TENANT.to_string(),
        negotiation_agent_process_id: urn(100),
        created_at: Utc::now().into(),
        direction: "inbound".into(),
        protocol: "dsp".into(),
        message_type: "ContractRequestMessage".into(),
        state_transition_from: "".into(),
        state_transition_to: "REQUESTED".into(),
        payload: json!({"@type": "ContractRequestMessage"}),
    }
}

pub fn offer_row(n: u32) -> offer::Model {
    offer::Model {
        id: urn(n),
        tenant_id: TENANT.to_string(),
        negotiation_agent_process_id: urn(100),
        negotiation_agent_message_id: urn(200),
        offer_id: format!("offer-{n}"),
        offer_content: json!({"permission": []}),
        created_at: Utc::now().into(),
    }
}

pub fn agreement_row(n: u32) -> agreement::Model {
    agreement::Model {
        id: urn(n),
        tenant_id: TENANT.to_string(),
        negotiation_agent_process_id: urn(100),
        negotiation_agent_message_id: urn(200),
        consumer_participant_id: "consumer".into(),
        provider_participant_id: "provider".into(),
        agreement_content: json!({"target": "urn:x"}),
        target: urn(300),
        state: "ACTIVE".into(),
        created_at: Utc::now().into(),
        updated_at: None,
    }
}
