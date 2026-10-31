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

//! Sends the queued `TransferStartMessage`s through the outbound flow, as the agent itself.

use std::sync::Arc;
use std::time::Duration;

use ymir::boot::workers::BackgroundWorker;
use common::dsp_common::DspCaller;
use common::flow::{FlowError, FlowRunner};
use ymir::types::oauth::UserInfo;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use ymir::errors::Outcome;

use crate::protocols::dsp::entities::context_rpc::{TransferRpcBody, TransferRpcTyped};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::services::auto_start::guard::AutoStartJobs;
use crate::protocols::dsp::services::rpc_flow::service::TransferRpcFlow;

/// Starts each queued process in its own task, retrying while the consumer is not ready.
pub struct AutoStartWorker {
    jobs: AutoStartJobs,
    runner: Arc<FlowRunner<TransferRpcFlow>>,
}

impl AutoStartWorker {
    /// Waits before each attempt: the consumer stores its process only after our ack.
    const DELAYS_MS: [u64; 8] = [400, 400, 800, 1600, 1600, 1600, 1600, 1600];

    pub fn new(jobs: AutoStartJobs, runner: Arc<FlowRunner<TransferRpcFlow>>) -> Self {
        Self { jobs, runner }
    }

    async fn start(runner: Arc<FlowRunner<TransferRpcFlow>>, process_id: String) {
        for (attempt, delay) in Self::DELAYS_MS.iter().enumerate() {
            tokio::time::sleep(Duration::from_millis(*delay)).await;
            match Self::send_start(&runner, &process_id).await {
                Ok(()) => {
                    tracing::info!(process_id, attempt = attempt + 1, "auto-start sent");
                    return;
                }
                Err(err) => {
                    tracing::warn!(process_id, attempt = attempt + 1, error = %err, "auto-start failed")
                }
            }
        }
        tracing::error!(process_id, "auto-start gave up");
    }

    async fn send_start(
        runner: &FlowRunner<TransferRpcFlow>,
        process_id: &str,
    ) -> Result<(), FlowError> {
        let message = TransferDSPMessageType::TransferStartMessage;
        let body = TransferRpcBody {
            id: Some(process_id.to_string()),
            ..TransferRpcBody::default()
        };
        let command = TransferRpcTyped::from_body(&message, &body).await?;
        runner
            .run(&DspCaller::User(UserInfo::system()), command)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for AutoStartWorker {
    fn name(&self) -> &'static str {
        "transfer-auto-start"
    }

    async fn run(mut self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        let mut starts = JoinSet::new();
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                job = self.jobs.0.recv() => match job {
                    Some(pid) => {
                        starts.spawn(Self::start(self.runner.clone(), pid));
                    }
                    None => token.cancelled().await,
                },
                Some(_) = starts.join_next(), if !starts.is_empty() => {}
            }
        }
        starts.shutdown().await;
        Ok(())
    }
}
