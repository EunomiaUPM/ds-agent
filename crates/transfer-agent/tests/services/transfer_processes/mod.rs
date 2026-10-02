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

//! TransferProcessService with mocked process and identifier repositories, split by operation.

mod batch;
mod create_delete;
mod edit;
mod get_one;
mod isolation;
mod list_filters;
mod list_paging;

use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use common::test_utils::scopes::TestScopes;
use transfer_agent::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use transfer_agent::entities::filters::TransferProcessFilter;
use urn::Urn;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use base64::Engine;
use chrono::{Duration, Utc};
use compact_str::CompactString;

use transfer_agent::data::repo::transfer_process::{
    MockTransferProcessRepoTrait, TransferProcessRepoErrors,
};
use transfer_agent::data::repo::transfer_process_identifier::{
    MockTransferIdentifierRepoTrait, TransferIdentifierRepoErrors,
};
use transfer_agent::entities::ids::{ParticipantId, TransferProcessId};
use transfer_agent::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use transfer_agent::entities::transfer_process::TransferProcess;
use transfer_agent::entities::transfer_process_identifier::TransferProcessIdentifier;
use transfer_agent::services::transfer_process::TransferProcessServiceTrait;
use transfer_agent::services::transfer_process::service::TransferProcessService;
use ymir::errors::RepoIntoErrors;

fn p_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{n:08x}-0000-0000-0000-000000000000")).expect("static URN")
}

fn empty_correlation() -> TransferCorrelation {
    TransferCorrelation::empty()
}

// make_process(n) produces a deterministic process: URN encodes n, and
// created_at/updated_at are offset by n so distinct processes have distinct timestamps.
fn make_process(n: u32) -> TransferProcess {
    let now = Utc::now();
    TransferProcess::rehydrate(
        TransferProcessId::new(p_urn(n)),
        "tenant-1".to_string(),
        TransferRole::Provider,
        now - Duration::seconds(n as i64 * 10),
        now - Duration::seconds(n as i64 * 5),
        0,
        ProtocolId::Dsp2024,
        ProtocolState(CompactString::from("STARTED")),
        StateMetadata::empty(),
        empty_correlation(),
        serde_json::json!({}),
        None,
    )
}

fn make_identifier(process_urn: Urn, key: &str, val: &str) -> TransferProcessIdentifier {
    TransferProcessIdentifier::new(process_urn, key.to_string(), Some(val.to_string()))
}

fn empty_filter() -> TransferProcessFilter {
    TransferProcessFilter {
        tenant_id: None,
        protocol: None,
        state: None,
        role: None,
        agreement_id: None,
        peer_participant_id: None,
        created_after: None,
        created_before: None,
    }
}

fn default_page() -> Page {
    Page::new(20, None)
}

fn make_new_cmd(identifiers: Option<HashMap<String, String>>) -> NewTransferProcessCommand {
    NewTransferProcessCommand {
        id: None,
        tenant_id: Some("tenant-1".to_string()),
        role: TransferRole::Consumer,
        protocol: ProtocolId::Dsp2024,
        initial_state: ProtocolState(CompactString::from("INITIATED")),
        initial_state_metadata: StateMetadata::empty(),
        callback_address: None,
        connector_instance_id: None,
        agreement_id: p_urn(99),
        peer_participant_id: ParticipantId::new(p_urn(100)),
        identifiers,
        properties: None,
    }
}

fn make_edit_cmd(
    state: Option<&str>,
    identifiers: Option<HashMap<String, String>>,
) -> EditTransferProcessCommand {
    EditTransferProcessCommand {
        state: state.map(|s| ProtocolState(CompactString::from(s))),
        state_metadata: None,
        identifiers,
        properties: None,
        error_details: None,
    }
}

fn make_svc(
    proc_repo: MockTransferProcessRepoTrait,
    id_repo: MockTransferIdentifierRepoTrait,
) -> TransferProcessService {
    TransferProcessService::new(Arc::new(proc_repo), Arc::new(id_repo))
}

fn io_err() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::from(std::io::ErrorKind::Other))
}

// get_all fires two repo calls concurrently via tokio::try_join!:
// get_all_transfer_processes (items) and count_transfer_processes (total).
// Identifiers are fetched in a third sequential call and merged into each view.

// These exercise the rules the service now owns on behalf of both transports.

// batch fetches all processes in a single repo call, then groups the flat
// identifier list by transfer_process_id before assembling each view.

// After persisting the process, create upserts each identifier individually.
// The returned view is assembled directly from cmd.identifiers — it does NOT
// re-fetch from the repo, unlike edit.

// edit puts the process, optionally upserts each identifier, then re-fetches ALL
// identifiers from the repo to build the view. This differs from create, where
// the view is built from the command without a subsequent fetch.
