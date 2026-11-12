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

//! Asynchronous, non-blocking signals emitted by the dataplane toward the hosting control plane.
//!
//! Keeping these types in `dataplane` avoids any cyclic crate dependency (`transfer-agent`
//! already depends on `dataplane`). When no control-plane worker is attached (e.g. isolated unit
//! tests), [`DataplaneSignalSender::noop`] drops signals silently at zero cost.

use tokio::sync::mpsc;
use tokio::sync::mpsc::error::{TryRecvError, TrySendError};
use urn::Urn;

/// Default capacity of the bounded signal channel between `dataplane` and `transfer-agent`.
pub const DEFAULT_SIGNAL_CHANNEL_CAPACITY: usize = 1024;

/// Lifecycle event observed on the data path or in an asynchronous driver hook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataplaneSignal {
    /// A `FINITE` transfer fulfilled its single-shot delivery and should move to `COMPLETED`.
    Completed { transfer_process_id: Urn },
    /// An active transfer suffered a permanent failure or exhausted `maxRetries` and should
    /// move to `TERMINATED`.
    Failed {
        transfer_process_id: Urn,
        code: String,
        reason: String,
    },
}

impl DataplaneSignal {
    pub fn transfer_process_id(&self) -> &Urn {
        match self {
            Self::Completed {
                transfer_process_id,
            }
            | Self::Failed {
                transfer_process_id,
                ..
            } => transfer_process_id,
        }
    }
}

/// Cloneable, non-blocking handle held by `HttpDataProxy` and `DataplaneManager`.
#[derive(Clone, Debug, Default)]
pub struct DataplaneSignalSender {
    tx: Option<mpsc::Sender<DataplaneSignal>>,
}

impl DataplaneSignalSender {
    /// Disconnected sender that discards every signal; used by default in tests.
    pub fn noop() -> Self {
        Self { tx: None }
    }

    /// Opens a bounded `(sender, receiver)` pair.
    pub fn open(capacity: usize) -> (Self, DataplaneSignalReceiver) {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        (
            Self { tx: Some(tx) },
            DataplaneSignalReceiver { rx },
        )
    }

    /// Emits `signal` without ever awaiting or blocking the HTTP data path.
    pub fn emit(&self, signal: DataplaneSignal) {
        let Some(tx) = &self.tx else {
            return;
        };
        match tx.try_send(signal) {
            Ok(()) => {}
            Err(TrySendError::Full(dropped)) => {
                tracing::warn!(
                    transfer_process_id = %dropped.transfer_process_id(),
                    "dataplane signal channel full; dropping signal"
                );
            }
            Err(TrySendError::Closed(dropped)) => {
                tracing::debug!(
                    transfer_process_id = %dropped.transfer_process_id(),
                    "dataplane signal receiver closed; ignoring signal"
                );
            }
        }
    }
}

/// Receiving end consumed by `DataplaneSignalWorker` in `transfer-agent`.
#[derive(Debug)]
pub struct DataplaneSignalReceiver {
    rx: mpsc::Receiver<DataplaneSignal>,
}

impl DataplaneSignalReceiver {
    pub async fn recv(&mut self) -> Option<DataplaneSignal> {
        self.rx.recv().await
    }

    pub fn try_recv(&mut self) -> Result<DataplaneSignal, TryRecvError> {
        self.rx.try_recv()
    }
}
