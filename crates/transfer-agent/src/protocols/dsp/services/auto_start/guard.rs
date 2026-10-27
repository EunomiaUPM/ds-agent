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

//! The guard that queues a provider process to start when its request asked for it.

use common::dsp_common::DspCaller;
use common::flow::FlowGuardTrait;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use ymir::errors::Outcome;

use crate::entities::protocol::TransferRole;
use crate::protocols::dsp::entities::context_dsp::TransferDspOutcome;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::services::dsp_flow::service::TransferDspFlow;

/// Ids of provider processes waiting for their `TransferStartMessage`.
pub struct AutoStartJobs(pub(crate) UnboundedReceiver<String>);

/// Queues the process of each new request that asked to be started.
#[derive(Clone)]
pub struct AutoStartGuard {
    queue: UnboundedSender<String>,
}

impl AutoStartGuard {
    /// The guard and the receiving end its worker drains.
    pub fn open() -> (Self, AutoStartJobs) {
        let (queue, jobs) = mpsc::unbounded_channel();
        (Self { queue }, AutoStartJobs(jobs))
    }
}

#[async_trait::async_trait]
impl FlowGuardTrait<TransferDspFlow> for AutoStartGuard {
    /// Only a request that opened the process: a replay must not start it twice.
    async fn after_commit(
        &self,
        _caller: &DspCaller,
        key: &TransferDSPMessageType,
        out: &TransferDspOutcome,
    ) -> Outcome<()> {
        if *key == TransferDSPMessageType::TransferRequestMessage
            && out.role == TransferRole::Provider
            && out.created
            && out.auto_start
        {
            let _ = self.queue.send(out.process_id.clone());
        }
        Ok(())
    }
}
