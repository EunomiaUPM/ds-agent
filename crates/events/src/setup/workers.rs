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

use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::services::event_bus::worker::RetryWorker;

// Handle for the background retry worker task providing cooperative shutdown.
pub struct RetryWorkerHandle {
    cancel_token: CancellationToken,
    handle: JoinHandle<()>,
}

impl RetryWorkerHandle {
    // Spawn a new background task running the RetryWorker.
    pub fn spawn(worker: Arc<RetryWorker>, cancel_token: CancellationToken) -> Self {
        let token_clone = cancel_token.clone();
        let handle = tokio::spawn(async move {
            worker.run(token_clone).await;
        });

        Self {
            cancel_token,
            handle,
        }
    }

    // Signal cooperative cancellation and await background task termination.
    pub async fn stop(self) {
        self.cancel_token.cancel();
        let _ = self.handle.await;
    }
}
