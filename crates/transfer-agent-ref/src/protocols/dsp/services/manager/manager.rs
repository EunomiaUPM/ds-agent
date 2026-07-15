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

use crate::protocols::dsp::entities::command::{TransferManagerCommand, TransferTransitionTrigger};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::services::manager::{TransferLifecycleStrategy, TransferResponse};
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_process::TransferProcessServiceTrait;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct DspManager {
    facades: Arc<dyn FacadeTrait>,
    transfer_service: Arc<dyn TransferProcessServiceTrait>,
    message_service: Arc<dyn TransferMessageServiceTrait>,
}

impl DspManager {
    pub fn new(
        facades: Arc<dyn FacadeTrait>,
        transfer_service: Arc<dyn TransferProcessServiceTrait>,
        message_service: Arc<dyn TransferMessageServiceTrait>,
    ) -> Self {
        Self {
            facades,
            transfer_service,
            message_service,
        }
    }

    /// Deterministic template: same phase order for every message; the strategy
    /// fills each phase. Takes `Arc<Self>` so the manager can hand a clone of
    /// itself to the strategies (re-entrancy: a strategy can `run` a follow-up
    /// command — e.g. a dataplane error → a fresh termination).
    pub async fn run(
        self: Arc<Self>,
        mut command: TransferManagerCommand,
    ) -> Outcome<TransferResponse> {
        let strategy = self.select(&command, Arc::clone(&self))?;
        strategy.validations(&mut command).await?;
        strategy.pre_hook(&mut command).await?;
        strategy.persist(&mut command).await?;
        strategy.send_to_peer(&command).await?;
        strategy.fire_events(&command).await?;
        strategy.post_hook(&mut command).await?;
        strategy.build_response(&command).await
    }

    /// The strategy axis is the incoming DSP message. Each strategy is built with
    /// the manager's injected deps (constructor injection), including a handle
    /// back to the manager for re-entrant runs.
    fn select(
        &self,
        command: &TransferManagerCommand,
        manager: Arc<DspManager>,
    ) -> Outcome<Box<dyn TransferLifecycleStrategy>> {
        use super::strategies::*;
        let deps = StrategyDeps {
            facades: self.facades.clone(),
            transfers: self.transfer_service.clone(),
            messages: self.message_service.clone(),
            manager,
        };
        let message = match &command.trigger {
            TransferTransitionTrigger::Dsp(m) => m,
            TransferTransitionTrigger::DataplaneSignal(_) => todo!(),
        };
        Ok(match message {
            TransferDSPMessageType::TransferRequestMessage => Box::new(RequestStrategy::new(deps)),
            TransferDSPMessageType::TransferStartMessage => Box::new(StartStrategy::new(deps)),
            TransferDSPMessageType::TransferSuspensionMessage => {
                Box::new(SuspendStrategy::new(deps))
            }
            TransferDSPMessageType::TransferCompletionMessage => {
                Box::new(CompleteStrategy::new(deps))
            }
            TransferDSPMessageType::TransferTerminationMessage => {
                Box::new(TerminateStrategy::new(deps))
            }
            other => {
                return Err(Errors::crazy(
                    format!("no lifecycle strategy for {other}"),
                    None,
                ));
            }
        })
    }
}
