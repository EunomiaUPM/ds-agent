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

//! The write side shared by the inbound and outbound flows; the dataplane is not part of it.

use std::sync::Arc;

use common::dsp_common::persistence::{DspPersistenceTrait, INITIAL_STATE};
use common::oauth::OwnerScope;
use ymir::types::oauth::UserInfo;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::commands::{
    EditTransferProcessCommand, NewTransferMessageCommand, NewTransferProcessCommand,
};
use crate::entities::protocol::{
    ProtocolId, ProtocolMessageType, ProtocolState, StateMetadata, TransferRole, CONSUMER_PID_KEY,
    PROVIDER_PID_KEY,
};
use crate::entities::transfer_message::Direction;
use crate::entities::transfer_process::TransferProcess;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::entities::state_change::{
    TransferMessageRecord, TransferRecordedMessage, TransferStateChange,
};
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;
use crate::setup::context::AppContext;

/// Writes transfer processes and their history through the agent's services.
#[derive(Clone)]
pub struct TransferDspPersistence {
    ctx: Arc<AppContext>,
}

impl TransferDspPersistence {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Who the protocol writes as: an in-process flow on a process it already authorized.
    pub fn actor() -> UserInfo {
        UserInfo::system()
    }

    /// Stores a new process as `user` and reads it back by the pid that is ours.
    pub async fn open_process(
        &self,
        user: &UserInfo,
        cmd: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcess> {
        self.ctx.processes().create(user, cmd).await?;
        let key = match cmd.role {
            TransferRole::Provider => PROVIDER_PID_KEY,
            _ => CONSUMER_PID_KEY,
        };
        let pid = cmd
            .identifiers
            .as_ref()
            .and_then(|ids| ids.get(key))
            .ok_or_else(|| Errors::crazy("a new transfer process needs our pid", None))?;
        self.reload(pid).await
    }

    /// The process stored under our own `id`, whoever owns it.
    pub async fn process_by_id(&self, id: &str) -> Outcome<Option<TransferProcess>> {
        let urn = id.parse::<Urn>().map_err(|_| {
            Errors::format(BadFormat::Received, format!("not a process id: {id}"), None)
        })?;
        self.ctx
            .repos
            .transfer_process_repo()
            .get_transfer_process_by_id(&OwnerScope::All, &urn)
            .await
    }

    /// The process again after a write.
    async fn reload(&self, pid: &str) -> Outcome<TransferProcess> {
        self.hydrate_process(pid).await?.ok_or_else(|| {
            Errors::crazy(
                format!("transfer process {pid} vanished after a write"),
                None,
            )
        })
    }

    /// The pid this connector minted for the process.
    fn own_pid(process: &TransferProcess) -> Outcome<String> {
        let correlation = process.correlation();
        match process.role() {
            TransferRole::Provider => correlation.provider_pid.clone(),
            _ => correlation.consumer_pid.clone(),
        }
        .ok_or_else(|| Errors::crazy("stored transfer process has no pid of ours", None))
    }

    /// The peer caused an inbound change, our own role an outbound one.
    fn caused_by(role: TransferRole, direction: Direction) -> TransferDSPStateAttribute {
        match (role, direction) {
            (TransferRole::Provider, Direction::Inbound)
            | (TransferRole::Consumer, Direction::Outbound) => {
                TransferDSPStateAttribute::ByConsumer
            }
            _ => TransferDSPStateAttribute::ByProvider,
        }
    }

    async fn write_message(
        &self,
        process: &TransferProcess,
        record: TransferMessageRecord,
        from: ProtocolState,
        to: ProtocolState,
    ) -> Outcome<()> {
        let cmd = NewTransferMessageCommand {
            id: None,
            transfer_process_id: process.id().clone(),
            visibility: None,
            owner: Some(process.owner().clone()),
            direction: record.direction,
            protocol: ProtocolId::Dsp2025_1,
            message_type: ProtocolMessageType(record.message.to_string().into()),
            state_transition_from: from,
            state_transition_to: to,
            envelope: record.envelope,
        };
        self.ctx.messages().create(&Self::actor(), &cmd).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DspPersistenceTrait for TransferDspPersistence {
    type Process = TransferProcess;
    type State = TransferDSPState;
    type Message = TransferMessageRecord;

    async fn hydrate_process(&self, pid: &str) -> Outcome<Option<TransferProcess>> {
        self.ctx
            .repos
            .transfer_process_repo()
            .get_transfer_process_by_key_value(&OwnerScope::All, pid)
            .await
    }

    async fn change_process_state(
        &self,
        process: &TransferProcess,
        change: TransferStateChange,
    ) -> Outcome<TransferProcess> {
        let from = process.state().clone();
        let to = ProtocolState(change.target.to_string().into());
        let attribute = Self::caused_by(process.role(), change.message.direction);
        let edit = EditTransferProcessCommand {
            state: Some(to.clone()),
            state_metadata: Some(StateMetadata {
                attribute: Some(attribute.to_string()),
                reason: change.reason,
                code: change.code,
            }),
            identifiers: None,
            properties: None,
            error_details: None,
        };
        self.ctx
            .processes()
            .edit(&Self::actor(), process.id().as_urn(), &edit)
            .await?;
        let updated = self.reload(&Self::own_pid(process)?).await?;
        self.write_message(&updated, change.message, from, to)
            .await?;
        Ok(updated)
    }

    async fn record_message(
        &self,
        process: &TransferProcess,
        message: TransferRecordedMessage,
    ) -> Outcome<()> {
        let from = ProtocolState(
            message
                .from
                .map_or_else(|| INITIAL_STATE.to_string(), |s| s.to_string())
                .into(),
        );
        let to = ProtocolState(message.to.to_string().into());
        self.write_message(process, message.message, from, to).await
    }
}
