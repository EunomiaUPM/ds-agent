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

//! Sends a scenario message through the outbound flow, as the agent itself.

use std::sync::Arc;

use common::dsp_common::persistence::DspPersistenceTrait;
use common::dsp_common::DspCaller;
use common::flow::{FlowError, FlowRunner};
use ymir::types::oauth::UserInfo;
use common::tck::TckPlayerTrait;
use ymir::errors::Errors;

use crate::protocols::dsp::entities::context_rpc::{TransferRpcBody, TransferRpcTyped};
use crate::protocols::dsp::entities::tck::TckMessage;
use crate::protocols::dsp::services::dsp_persistence::service::TransferDspPersistence;
use crate::protocols::dsp::services::rpc_flow::service::TransferRpcFlow;

/// Plays TCK steps on the outbound runner.
pub struct TransferTckPlayer {
    runner: Arc<FlowRunner<TransferRpcFlow>>,
    persistence: TransferDspPersistence,
}

impl TransferTckPlayer {
    pub fn new(
        runner: Arc<FlowRunner<TransferRpcFlow>>,
        persistence: TransferDspPersistence,
    ) -> Self {
        Self {
            runner,
            persistence,
        }
    }
}

#[async_trait::async_trait]
impl TckPlayerTrait<TckMessage> for TransferTckPlayer {
    async fn send(&self, pid: &str, message: TckMessage) -> Result<(), FlowError> {
        let process = self.persistence.hydrate_process(pid).await?.ok_or_else(|| {
            Errors::missing_resource(pid.to_string(), "transfer process not found", None)
        })?;
        let body = TransferRpcBody {
            id: Some(process.id().as_urn().to_string()),
            ..TransferRpcBody::default()
        };
        let command = TransferRpcTyped::from_body(&message.message_type(), &body).await?;
        let agent = DspCaller::User(UserInfo::system());
        self.runner.run(&agent, command).await?;
        Ok(())
    }
}
