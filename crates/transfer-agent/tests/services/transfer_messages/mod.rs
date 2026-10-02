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

//! TransferMessageService with a mocked repository, split by operation.

mod by_process;
mod isolation;
mod list;
mod single;

use common::query::{Page, Sort};
use common::test_utils::scopes::TestScopes;
use transfer_agent::entities::commands::NewTransferMessageCommand;
use transfer_agent::entities::filters::TransferMessageFilter;
use urn::Urn;

use std::str::FromStr;
use std::sync::Arc;

use base64::Engine;
use chrono::{Duration, Utc};
use compact_str::CompactString;

use transfer_agent::data::repo::transfer_message::{
    MockTransferMessageRepoTrait, TransferMessageRepoErrors,
};
use transfer_agent::entities::ids::{MessageId, TransferProcessId};
use transfer_agent::entities::message_envelope::MessageEnvelope;
use transfer_agent::entities::protocol::{ProtocolId, ProtocolMessageType, ProtocolState};
use transfer_agent::entities::transfer_message::{Direction, TransferMessage};
use transfer_agent::services::transfer_message::TransferMessageServiceTrait;
use transfer_agent::services::transfer_message::service::TransferMessageService;
use ymir::errors::RepoIntoErrors;

fn p_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{n:08x}-0000-0000-0000-000000000000")).expect("static URN")
}

fn make_envelope() -> MessageEnvelope {
    MessageEnvelope {
        canonical_form: None,
        canonical_hash: None,
        payload: serde_json::json!({}),
    }
}

fn make_message(n: u32) -> TransferMessage {
    TransferMessage {
        id: MessageId::new(p_urn(n + 1000)),
        transfer_process_id: TransferProcessId::new(p_urn(1)),
        tenant_id: "tenant-1".to_string(),
        direction: Direction::Inbound,
        protocol: ProtocolId::Dsp2024,
        message_type: ProtocolMessageType(CompactString::from("TransferRequestMessage")),
        state_transition_from: "INITIAL".to_string(),
        state_transition_to: "STARTED".to_string(),
        envelope: make_envelope(),
        occurred_at: Utc::now() - Duration::seconds(n as i64 * 10),
    }
}

fn empty_filter() -> TransferMessageFilter {
    TransferMessageFilter {
        tenant_id: None,
        direction: None,
        protocol: None,
        state_transition_to: None,
        created_after: None,
        created_before: None,
    }
}

fn default_page() -> Page {
    Page::new(20, None)
}

fn make_cmd() -> NewTransferMessageCommand {
    NewTransferMessageCommand {
        id: None,
        transfer_process_id: TransferProcessId::new(p_urn(1)),
        tenant_id: Some("tenant-1".to_string()),
        direction: Direction::Inbound,
        protocol: ProtocolId::Dsp2024,
        message_type: ProtocolMessageType(CompactString::from("TransferRequestMessage")),
        state_transition_from: ProtocolState(CompactString::from("INITIAL")),
        state_transition_to: ProtocolState(CompactString::from("STARTED")),
        envelope: make_envelope(),
    }
}

fn make_svc(repo: MockTransferMessageRepoTrait) -> TransferMessageService {
    TransferMessageService::new(Arc::new(repo))
}

fn io_err() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::from(std::io::ErrorKind::Other))
}

// Like the process service, get_all fires get_all_transfer_messages and
// count_transfer_messages concurrently via tokio::try_join!. The cursor for
// messages is always based on occurred_at, regardless of the sort field.

// get_all_by_process delegates to get_messages_by_process_id for the items, but
// still calls count_transfer_messages (without process_id) for the total. The
// cursor logic and filter pass-through are identical to get_all.

// get_one converts Option::None from the repo into a not-found error using
// TransferMessageRepoErrors::TransferMessageNotFound.

// create delegates entirely to the repo and assembles the view from whatever
// the repo returns — not from the command fields.
