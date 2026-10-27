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
use ymir::errors::{Errors, Outcome};

use crate::protocols::dsp::entities::context_rpc::{NegotiationRpcBody, NegotiationRpcTyped};
use crate::protocols::dsp::entities::message_types::NegotiationRpcMessage;
use crate::protocols::dsp::entities::tck::TckMessage;
use crate::protocols::dsp::services::dsp_persistence::service::NegotiationDspPersistence;
use crate::protocols::dsp::services::rpc_flow::service::NegotiationRpcFlow;

/// Plays TCK steps on the outbound runner; a counter message repeats the last offer.
pub struct NegotiationTckPlayer {
    runner: Arc<FlowRunner<NegotiationRpcFlow>>,
    persistence: NegotiationDspPersistence,
}

impl NegotiationTckPlayer {
    pub fn new(
        runner: Arc<FlowRunner<NegotiationRpcFlow>>,
        persistence: NegotiationDspPersistence,
    ) -> Self {
        Self {
            runner,
            persistence,
        }
    }

    /// The last offer of the negotiation behind `pid`, as stored.
    async fn last_offer(&self, pid: &str) -> Outcome<Option<serde_json::Value>> {
        let Some(process) = self.persistence.hydrate_process(pid).await? else {
            return Ok(None);
        };
        Ok(self
            .persistence
            .last_offer(&process)
            .await?
            .map(|offer| offer.offer_content))
    }
}

#[async_trait::async_trait]
impl TckPlayerTrait<TckMessage> for NegotiationTckPlayer {
    async fn send(&self, pid: &str, message: TckMessage) -> Result<(), FlowError> {
        let rpc = message.rpc_message();
        let offer = match rpc {
            NegotiationRpcMessage::Request | NegotiationRpcMessage::Offer => {
                self.last_offer(pid).await?
            }
            _ => None,
        };
        let process = self.persistence.hydrate_process(pid).await?.ok_or_else(|| {
            Errors::missing_resource(pid.to_string(), "negotiation not found", None)
        })?;
        let body = NegotiationRpcBody {
            id: Some(process.id()?.to_string()),
            offer,
            ..NegotiationRpcBody::default()
        };
        let command = NegotiationRpcTyped::from_body(rpc, &body).await?;
        let agent = DspCaller::User(UserInfo::system());
        self.runner.run(&agent, command).await?;
        Ok(())
    }
}
