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

//! Consumes `DataplaneSignal` events emitted by `HttpDataProxy` and `DataplaneManager`,
//! driving `TransferCompletionMessage` or `TransferTerminationMessage` through the outbound
//! DSP RPC flow with bounded retries and local fallback termination when the peer is unreachable.

use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::dsp_common::DspCaller;
use common::flow::{FlowError, FlowRunner};
use dataplane::{DataplaneSignal, DataplaneSignalReceiver};
use serde_json::json;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::commands::EditTransferProcessCommand;
use crate::entities::protocol::{ProtocolState, StateMetadata, TransferDirection, TransferRole};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, DIRECTION_PROPERTY,
};
use crate::protocols::dsp::entities::context_rpc::{TransferRpcBody, TransferRpcTyped};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute;
use crate::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;
use crate::protocols::dsp::services::dsp_persistence::service::TransferDspPersistence;
use crate::protocols::dsp::services::rpc_flow::service::TransferRpcFlow;
use crate::setup::context::AppContext;

/// Releases a `transfer_process_id` from the in-flight deduplication set when dropped.
struct InFlightGuard {
    in_flight: Arc<Mutex<HashSet<String>>>,
    process_id: String,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = self.in_flight.lock() {
            set.remove(&self.process_id);
        }
    }
}

/// Background worker translating `DataplaneSignal` events into outbound DSP 2025-1 transitions.
pub struct DataplaneSignalWorker {
    signals: DataplaneSignalReceiver,
    app: Arc<AppContext>,
    runner: Arc<FlowRunner<TransferRpcFlow>>,
    retry_delays_ms: Vec<u64>,
}

impl DataplaneSignalWorker {
    /// Default retry delays after the immediate initial attempt.
    const DEFAULT_RETRY_DELAYS_MS: [u64; 4] = [150, 300, 600, 1200];

    pub fn new(
        signals: DataplaneSignalReceiver,
        app: Arc<AppContext>,
        runner: Arc<FlowRunner<TransferRpcFlow>>,
    ) -> Self {
        Self {
            signals,
            app,
            runner,
            retry_delays_ms: Self::DEFAULT_RETRY_DELAYS_MS.to_vec(),
        }
    }

    /// Overrides the retry backoff schedule (primarily for deterministic unit tests).
    pub fn with_retry_delays(mut self, delays_ms: Vec<u64>) -> Self {
        self.retry_delays_ms = delays_ms;
        self
    }

    /// Processes a single dataplane signal end-to-end (useful in unit tests without spawning).
    pub async fn process_signal(
        app: Arc<AppContext>,
        runner: Arc<FlowRunner<TransferRpcFlow>>,
        signal: DataplaneSignal,
        retry_delays_ms: &[u64],
    ) {
        let persistence = TransferDspPersistence::new(app.clone());
        let pid_str = signal.transfer_process_id().to_string();

        let total_attempts = retry_delays_ms.len() + 1;
        for attempt in 0..total_attempts {
            if attempt > 0 {
                let delay = retry_delays_ms[attempt - 1];
                if delay > 0 {
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }

            match Self::current_state(&persistence, &pid_str).await {
                Ok(None) => {
                    tracing::debug!(
                        process_id = %pid_str,
                        "ignoring dataplane signal for unknown or deleted transfer process"
                    );
                    return;
                }
                Ok(Some(TransferDSPState::COMPLETED | TransferDSPState::TERMINATED)) => {
                    tracing::debug!(
                        process_id = %pid_str,
                        "ignoring dataplane signal because transfer process is already terminal"
                    );
                    return;
                }
                Ok(Some(state))
                    if matches!(signal, DataplaneSignal::Completed { .. })
                        && state != TransferDSPState::STARTED =>
                {
                    tracing::warn!(
                        process_id = %pid_str,
                        %state,
                        "ignoring dataplane completion signal because transfer process is not STARTED"
                    );
                    return;
                }
                Ok(Some(_)) => {}
                Err(err) => {
                    tracing::warn!(
                        process_id = %pid_str,
                        error = %err,
                        "failed to inspect transfer process state before dispatching signal"
                    );
                }
            }

            match Self::dispatch_outbound(&runner, &signal).await {
                Ok(()) => {
                    tracing::info!(
                        process_id = %pid_str,
                        attempt = attempt + 1,
                        "dataplane signal dispatched over DSP"
                    );
                    return;
                }
                Err(err) => {
                    tracing::warn!(
                        process_id = %pid_str,
                        attempt = attempt + 1,
                        total_attempts,
                        error = %err,
                        "dataplane signal outbound DSP dispatch failed"
                    );
                }
            }
        }

        // If a Failed signal could not be delivered to the peer after all retries, force local
        // termination and dataplane teardown so neither state nor subscriptions/secrets leak.
        if let DataplaneSignal::Failed { code, reason, .. } = signal {
            Self::force_local_termination(&app, &persistence, &pid_str, &code, &reason).await;
        } else {
            tracing::error!(
                process_id = %pid_str,
                "dataplane completion signal exhausted retries without peer acknowledgement"
            );
        }
    }

    async fn current_state(
        persistence: &TransferDspPersistence,
        pid_str: &str,
    ) -> Outcome<Option<TransferDSPState>> {
        let Some(process) = persistence.process_by_id(pid_str).await? else {
            return Ok(None);
        };
        let state = TransferDSPState::from_str(process.state().0.as_str())?;
        Ok(Some(state))
    }

    async fn dispatch_outbound(
        runner: &FlowRunner<TransferRpcFlow>,
        signal: &DataplaneSignal,
    ) -> Result<(), FlowError> {
        let (message, body) = match signal {
            DataplaneSignal::Completed {
                transfer_process_id,
            } => (
                TransferDSPMessageType::TransferCompletionMessage,
                TransferRpcBody {
                    id: Some(transfer_process_id.to_string()),
                    ..TransferRpcBody::default()
                },
            ),
            DataplaneSignal::Failed {
                transfer_process_id,
                code,
                reason,
            } => (
                TransferDSPMessageType::TransferTerminationMessage,
                TransferRpcBody {
                    id: Some(transfer_process_id.to_string()),
                    code: Some(code.clone()),
                    reason: Some(vec![reason.clone()]),
                    ..TransferRpcBody::default()
                },
            ),
        };
        let command = TransferRpcTyped::from_body(&message, &body).await?;
        runner
            .run(&DspCaller::User(UserInfo::system()), command)
            .await?;
        Ok(())
    }

    async fn force_local_termination(
        app: &AppContext,
        persistence: &TransferDspPersistence,
        pid_str: &str,
        code: &str,
        reason: &str,
    ) {
        let process = match persistence.process_by_id(pid_str).await {
            Ok(Some(process)) => process,
            Ok(None) => return,
            Err(err) => {
                tracing::error!(
                    process_id = %pid_str,
                    error = %err,
                    "failed to load transfer process for forced local termination"
                );
                return;
            }
        };

        if matches!(
            TransferDSPState::from_str(process.state().0.as_str()),
            Ok(TransferDSPState::COMPLETED | TransferDSPState::TERMINATED)
        ) {
            return;
        }

        let attribute = match process.role() {
            TransferRole::Provider => TransferDSPStateAttribute::ByProvider,
            _ => TransferDSPStateAttribute::ByConsumer,
        };
        let edit = EditTransferProcessCommand {
            state: Some(ProtocolState(TransferDSPState::TERMINATED.to_string().into())),
            state_metadata: Some(StateMetadata {
                attribute: Some(attribute.to_string()),
                reason: Some(vec![reason.to_string()]),
                code: Some(code.to_string()),
            }),
            identifiers: None,
            properties: None,
            error_details: Some(json!({
                "forcedLocalTermination": true,
                "code": code,
                "reason": reason,
            })),
        };

        if let Err(err) = app
            .processes()
            .edit(&TransferDspPersistence::actor(), process.id().as_urn(), &edit)
            .await
        {
            tracing::error!(
                process_id = %pid_str,
                error = %err,
                "failed to persist forced local TERMINATED state"
            );
        } else {
            tracing::warn!(
                process_id = %pid_str,
                code,
                reason,
                "peer unreachable after retries; forced local TERMINATED transition"
            );
        }

        let transfer_direction = process
            .properties()
            .get(DIRECTION_PROPERTY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(TransferDirection::Pull);
        let hooks = DataplaneHookContext {
            process_id: process.id().as_urn().clone(),
            owner: process.owner().clone(),
            role: process.role(),
            transfer_direction,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            resolved_data_address: None,
            data_address: None,
            is_restart: false,
        };
        let _ = app.ports.dataplane.on_transfer_termination_pre(&hooks).await;
        let _ = app.ports.dataplane.on_transfer_termination_post(&hooks).await;
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for DataplaneSignalWorker {
    fn name(&self) -> &'static str {
        "transfer-dataplane-signals"
    }

    async fn run(mut self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        let mut tasks = JoinSet::new();
        let in_flight: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        let retry_delays: Arc<[u64]> = self.retry_delays_ms.into();

        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                maybe_signal = self.signals.recv() => match maybe_signal {
                    Some(signal) => {
                        let pid_str = signal.transfer_process_id().to_string();
                        let acquired = in_flight
                            .lock()
                            .map(|mut set| set.insert(pid_str.clone()))
                            .unwrap_or(true);
                        if !acquired {
                            tracing::debug!(
                                process_id = %pid_str,
                                "coalescing duplicate dataplane signal while transition is in flight"
                            );
                            continue;
                        }
                        let guard = InFlightGuard {
                            in_flight: in_flight.clone(),
                            process_id: pid_str,
                        };
                        let app = self.app.clone();
                        let runner = self.runner.clone();
                        let delays = retry_delays.clone();
                        tasks.spawn(async move {
                            let _guard = guard;
                            Self::process_signal(app, runner, signal, &delays).await;
                        });
                    }
                    None => token.cancelled().await,
                },
                Some(_) = tasks.join_next(), if !tasks.is_empty() => {}
            }
        }
        tasks.shutdown().await;
        Ok(())
    }
}
