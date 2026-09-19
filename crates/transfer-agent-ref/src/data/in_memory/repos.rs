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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::data::repo::transfer_message::{TransferMessageRepoErrors, TransferMessageRepoTrait};
use crate::data::repo::transfer_process::{TransferProcessRepoErrors, TransferProcessRepoTrait};
use crate::data::repo::transfer_process_identifier::TransferIdentifierRepoTrait;
use crate::entities::commands::{
    EditTransferProcessCommand, NewTransferMessageCommand, NewTransferProcessCommand,
};
use crate::entities::filters::{TransferMessageFilter, TransferProcessFilter};
use crate::entities::ids::TransferProcessId;
use crate::entities::protocol::TransferCorrelation;
use crate::entities::transfer_message::TransferMessage;
use crate::entities::transfer_process::TransferProcess;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use base64::Engine;
use chrono::{DateTime, Utc};
use common::query::{Cursor, Page, Sort};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

// Shared store types ────────────────────────────────────────────────────────

#[allow(dead_code)]
type ProcessStore = Arc<Mutex<HashMap<String, TransferProcess>>>;
#[allow(dead_code)]
type MessageStore = Arc<Mutex<HashMap<String, TransferMessage>>>;
#[allow(dead_code)]
type IdentifierStore = Arc<Mutex<HashMap<(String, String), TransferProcessIdentifier>>>;

// TransferProcessRepo ───────────────────────────────────────────────────────

#[allow(dead_code)]
pub(crate) struct InMemoryTransferProcessRepo {
    processes: ProcessStore,
    identifiers: IdentifierStore,
}

#[allow(dead_code)]
impl InMemoryTransferProcessRepo {
    pub fn new(processes: ProcessStore, identifiers: IdentifierStore) -> Self {
        Self {
            processes,
            identifiers,
        }
    }
}

#[async_trait::async_trait]
impl TransferProcessRepoTrait for InMemoryTransferProcessRepo {
    async fn get_all_transfer_processes(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferProcess>> {
        let store = self.processes.lock().unwrap();
        let cursor_dt = decode_cursor(page.cursor.as_deref());

        let mut items: Vec<TransferProcess> = store
            .values()
            .filter(|p| {
                if !matches_process_filter(p, filters) {
                    return false;
                }
                if let Some(cursor) = cursor_dt {
                    match sort {
                        Sort::CreatedAtAsc => {
                            if p.created_at() <= cursor {
                                return false;
                            }
                        }
                        Sort::UpdatedAtAsc => {
                            if p.updated_at() <= cursor {
                                return false;
                            }
                        }
                        Sort::UpdatedAtDesc => {
                            if p.updated_at() >= cursor {
                                return false;
                            }
                        }
                        _ => {
                            if p.created_at() >= cursor {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        match sort {
            Sort::CreatedAtAsc => {
                items.sort_by(|a, b| {
                    a.created_at()
                        .cmp(&b.created_at())
                        .then_with(|| a.id().to_string().cmp(&b.id().to_string()))
                });
            }
            Sort::UpdatedAtAsc => {
                items.sort_by(|a, b| {
                    a.updated_at()
                        .cmp(&b.updated_at())
                        .then_with(|| a.id().to_string().cmp(&b.id().to_string()))
                });
            }
            Sort::UpdatedAtDesc => {
                items.sort_by(|a, b| {
                    b.updated_at()
                        .cmp(&a.updated_at())
                        .then_with(|| b.id().to_string().cmp(&a.id().to_string()))
                });
            }
            _ => {
                items.sort_by(|a, b| {
                    b.created_at()
                        .cmp(&a.created_at())
                        .then_with(|| b.id().to_string().cmp(&a.id().to_string()))
                });
            }
        }

        items.truncate(page.limit as usize);
        Ok(items)
    }

    async fn count_transfer_processes(&self, filters: &TransferProcessFilter) -> Outcome<u64> {
        let store = self.processes.lock().unwrap();
        let count = store
            .values()
            .filter(|p| matches_process_filter(p, filters))
            .count() as u64;
        Ok(count)
    }

    async fn get_batch_transfer_processes(
        &self,
        tenant_id: &str,
        ids: &[Urn],
    ) -> Outcome<Vec<TransferProcess>> {
        let store = self.processes.lock().unwrap();
        let id_strs: Vec<String> = ids.iter().map(|u| u.to_string()).collect();
        Ok(store
            .iter()
            .filter(|(k, v)| {
                id_strs.contains(k) && v.tenant_id().as_str() == tenant_id
            })
            .map(|(_, v)| v.clone())
            .collect())
    }

    async fn get_transfer_process_by_id(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>> {
        let store = self.processes.lock().unwrap();
        let p = store.get(&id.to_string()).cloned();
        Ok(p.filter(|p| p.tenant_id().as_str() == tenant_id))
    }


    async fn get_transfer_process_by_key_value(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>> {
        let target = id.to_string();
        let pid = {
            let idents = self.identifiers.lock().unwrap();
            idents
                .iter()
                .find(|(_, v)| {
                    v.value.as_deref() == Some(&target)
                        && tenant_id.as_deref().map_or(true, |tid| v.tenant_id.as_str() == tid)
                })
                .map(|((pid, _), _)| pid.clone())
        };
        let process = pid.and_then(|pid| {
            let store = self.processes.lock().unwrap();
            let p = store.get(&pid).cloned();
            if let Some(tid) = &tenant_id {
                p.filter(|p| p.tenant_id().as_str() == tid.as_str())
            } else {
                p
            }
        });
        Ok(process)
    }

    async fn create_transfer_process(
        &self,
        cmd: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        let process = process_from_cmd(cmd)?;
        self.processes
            .lock()
            .unwrap()
            .insert(process.id().to_string(), process.clone());
        Ok(process)
    }

    async fn put_transfer_process(
        &self,
        tenant_id: &str,
        id: &Urn,
        edit_model: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        let process = {
            let mut store = self.processes.lock().unwrap();
            let p = store
                .get_mut(&id.to_string())
                .ok_or_else(|| TransferProcessRepoErrors::TransferProcessNotFound.into_errors())?;
            if p.tenant_id().as_str() != tenant_id {
                return Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors());
            }
            p.apply_edit(edit_model.clone());
            p.clone()
        }; // MutexGuard dropped here
        Ok(process)
    }

    async fn delete_transfer_process(&self, tenant_id: &str, id: &Urn) -> Outcome<()> {
        let mut store = self.processes.lock().unwrap();
        if let Some(p) = store.get(&id.to_string()) {
            if p.tenant_id().as_str() != tenant_id {
                return Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors());
            }
        } else {
            return Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors());
        }
        let removed = store.remove(&id.to_string());
        if removed.is_none() {
            return Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors());
        }
        self.identifiers
            .lock()
            .unwrap()
            .retain(|(pid, _), _| pid != &id.to_string());
        Ok(())
    }
}

// TransferMessageRepo ───────────────────────────────────────────────────────

#[allow(dead_code)]
pub(crate) struct InMemoryTransferMessageRepo {
    messages: MessageStore,
}

#[allow(dead_code)]
impl InMemoryTransferMessageRepo {
    pub fn new(messages: MessageStore) -> Self {
        Self { messages }
    }
}

#[async_trait::async_trait]
impl TransferMessageRepoTrait for InMemoryTransferMessageRepo {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let store = self.messages.lock().unwrap();
        Ok(filter_messages(store.values(), None, filters, page, sort))
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let store = self.messages.lock().unwrap();
        Ok(filter_messages(
            store.values(),
            Some(process_id),
            filters,
            page,
            sort,
        ))
    }

    async fn count_transfer_messages(&self, filters: &TransferMessageFilter) -> Outcome<u64> {
        let store = self.messages.lock().unwrap();
        let count = store
            .values()
            .filter(|m| matches_message_filter(m, None, filters))
            .count() as u64;
        Ok(count)
    }

    async fn get_transfer_message_by_id(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<TransferMessage>> {
        let store = self.messages.lock().unwrap();
        let m = store.get(&id.to_string()).cloned();
        Ok(m.filter(|m| m.tenant_id().as_str() == tenant_id))
    }

    async fn create_transfer_message(
        &self,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessage> {
        let msg = TransferMessage::from_cmd(cmd)?;
        self.messages
            .lock()
            .unwrap()
            .insert(msg.id().to_string(), msg.clone());
        Ok(msg)
    }

    async fn delete_transfer_message(&self, tenant_id: &str, id: &Urn) -> Outcome<()> {
        let mut store = self.messages.lock().unwrap();
        if let Some(m) = store.get(&id.to_string()) {
            if m.tenant_id().as_str() != tenant_id {
                return Err(TransferMessageRepoErrors::TransferMessageNotFound.into_errors());
            }
        } else {
            return Err(TransferMessageRepoErrors::TransferMessageNotFound.into_errors());
        }
        let removed = store.remove(&id.to_string());
        if removed.is_none() {
            return Err(TransferMessageRepoErrors::TransferMessageNotFound.into_errors());
        }
        Ok(())
    }
}

// TransferIdentifierRepo ────────────────────────────────────────────────────

#[allow(dead_code)]
pub(crate) struct InMemoryTransferIdentifierRepo {
    identifiers: IdentifierStore,
}

#[allow(dead_code)]
impl InMemoryTransferIdentifierRepo {
    pub fn new(identifiers: IdentifierStore) -> Self {
        Self { identifiers }
    }
}

#[async_trait::async_trait]
impl TransferIdentifierRepoTrait for InMemoryTransferIdentifierRepo {
    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferProcessIdentifier>> {
        let pid = process_id.to_string();
        Ok(self
            .identifiers
            .lock()
            .unwrap()
            .iter()
            .filter(|((p, _), _)| p == &pid)
            .map(|(_, v)| v.clone())
            .collect())
    }

    async fn get_identifiers_by_batch_process_id(
        &self,
        process_id_batch: &[Urn],
    ) -> Outcome<Vec<TransferProcessIdentifier>> {
        let pids: Vec<String> = process_id_batch.iter().map(|u| u.to_string()).collect();
        Ok(self
            .identifiers
            .lock()
            .unwrap()
            .iter()
            .filter(|((p, _), _)| pids.contains(p))
            .map(|(_, v)| v.clone())
            .collect())
    }

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> Outcome<Option<TransferProcessIdentifier>> {
        let k = (process_id.to_string(), key.to_string());
        Ok(self.identifiers.lock().unwrap().get(&k).cloned())
    }

    async fn upsert_identifier(
        &self,
        process_id: &Urn,
        identifier: &TransferProcessIdentifier,
    ) -> Outcome<TransferProcessIdentifier> {
        let k = (process_id.to_string(), identifier.key.clone());
        let ident = TransferProcessIdentifier {
            tenant_id: identifier.tenant_id.clone(),
            transfer_process_id: process_id.clone(),
            key: identifier.key.clone(),
            value: identifier.value.clone(),
        };
        self.identifiers.lock().unwrap().insert(k, ident.clone());
        Ok(ident)
    }

    async fn delete_identifier(&self, process_id: &Urn, key: &str) -> Outcome<()> {
        let k = (process_id.to_string(), key.to_string());
        self.identifiers.lock().unwrap().remove(&k);
        Ok(())
    }
}

// Helpers ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn decode_cursor(cursor: Option<&str>) -> Option<DateTime<Utc>> {
    cursor.and_then(|c| Cursor::decode_utc_timestamp(c).ok())
}

#[allow(dead_code)]
fn matches_process_filter(p: &TransferProcess, filters: &TransferProcessFilter) -> bool {
    if let Some(tid) = &filters.tenant_id {
        if p.tenant_id().as_str() != tid.as_str() {
            return false;
        }
    }
    if let Some(proto) = &filters.protocol {
        if p.protocol() != proto {
            return false;
        }
    }
    if let Some(state) = &filters.state {
        if p.state() != state {
            return false;
        }
    }
    if let Some(role) = &filters.role {
        if p.role() != *role {
            return false;
        }
    }
    if let Some(aid) = &filters.agreement_id {
        if p.correlation().agreement_id.as_ref() != Some(aid) {
            return false;
        }
    }
    if let Some(peer) = &filters.peer_participant_id {
        let stored = p
            .correlation()
            .peer_participant_id
            .as_ref()
            .map(|x| x.as_urn());
        if stored != Some(peer.as_urn()) {
            return false;
        }
    }
    if let Some(after) = filters.created_after {
        if p.created_at() <= after {
            return false;
        }
    }
    if let Some(before) = filters.created_before {
        if p.created_at() >= before {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn matches_message_filter(
    m: &TransferMessage,
    process_id: Option<&Urn>,
    filters: &TransferMessageFilter,
) -> bool {
    if let Some(tid) = &filters.tenant_id {
        if m.tenant_id().as_str() != tid.as_str() {
            return false;
        }
    }
    if let Some(pid) = process_id {
        if m.transfer_process_id().as_urn() != pid {
            return false;
        }
    }
    if let Some(dir) = &filters.direction {
        if m.direction() != *dir {
            return false;
        }
    }
    if let Some(proto) = &filters.protocol {
        if m.protocol() != proto {
            return false;
        }
    }
    if let Some(state) = &filters.state_transition_to {
        if m.state_transition_to() != state.0.as_str() {
            return false;
        }
    }
    if let Some(after) = filters.created_after {
        if m.occurred_at() <= after {
            return false;
        }
    }
    if let Some(before) = filters.created_before {
        if m.occurred_at() >= before {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn filter_messages<'a>(
    values: impl Iterator<Item = &'a TransferMessage>,
    process_id: Option<&Urn>,
    filters: &TransferMessageFilter,
    page: &Page,
    sort: &Sort,
) -> Vec<TransferMessage> {
    let cursor_dt = decode_cursor(page.cursor.as_deref());

    let mut items: Vec<TransferMessage> = values
        .filter(|m| {
            if !matches_message_filter(m, process_id, filters) {
                return false;
            }
            if let Some(cursor) = cursor_dt {
                match sort {
                    Sort::CreatedAtAsc => {
                        if m.occurred_at() <= cursor {
                            return false;
                        }
                    }
                    _ => {
                        if m.occurred_at() >= cursor {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .cloned()
        .collect();

    match sort {
        Sort::CreatedAtAsc => items.sort_by(|a, b| {
            a.occurred_at()
                .cmp(&b.occurred_at())
                .then_with(|| a.id.to_string().cmp(&b.id.to_string()))
        }),
        _ => items.sort_by(|a, b| {
            b.occurred_at()
                .cmp(&a.occurred_at())
                .then_with(|| b.id.to_string().cmp(&a.id.to_string()))
        }),
    }

    items.truncate(page.limit as usize);
    items
}

#[allow(dead_code, clippy::result_large_err)]
fn process_from_cmd(cmd: &NewTransferProcessCommand) -> Outcome<TransferProcess> {
    let id = cmd.id.clone().unwrap_or_else(TransferProcessId::generate);
    let tenant_id = cmd.tenant_id.clone().ok_or_else(|| {
        ymir::errors::Errors::crazy("tenant_id must be resolved before reaching the repo", None)
    })?;
    let now = chrono::Utc::now();
    let correlation = TransferCorrelation {
        identifiers: std::collections::HashMap::new(),
        consumer_pid: None,
        provider_pid: None,
        agreement_id: Some(cmd.agreement_id.clone()),
        callback_address: cmd.callback_address.clone(),
        peer_participant_id: Some(cmd.peer_participant_id.clone()),
    };
    Ok(TransferProcess::rehydrate(
        id,
        tenant_id,
        cmd.role,
        now,
        now,
        0,
        cmd.protocol.clone(),
        cmd.initial_state.clone(),
        cmd.initial_state_metadata.clone(),
        correlation,
        cmd.properties.clone().unwrap_or(serde_json::json!({})),
        None,
    ))
}
