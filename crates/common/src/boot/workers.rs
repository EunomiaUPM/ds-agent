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

//! Long-running background tasks supervised as one set with coordinated shutdown.

use std::collections::HashMap;
use std::time::Duration;

use tokio::task::{Id, JoinSet};
use tokio_util::sync::CancellationToken;
use ymir::errors::{Errors, Outcome};

/// A task that runs for the whole process lifetime until its token is cancelled.
#[async_trait::async_trait]
pub trait BackgroundWorker: Send + 'static {
    fn name(&self) -> &'static str;

    /// Consumes the worker; returning before cancellation means it stopped on its own.
    async fn run(self: Box<Self>, token: CancellationToken) -> Outcome<()>;
}

/// How a supervised worker ended.
pub struct WorkerExit {
    pub name: &'static str,
    pub result: Outcome<()>,
}

/// Every background worker of the process, sharing one cancellation token.
pub struct WorkerSet {
    token: CancellationToken,
    tasks: JoinSet<Outcome<()>>,
    names: HashMap<Id, &'static str>,
}

impl WorkerSet {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            token,
            tasks: JoinSet::new(),
            names: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, worker: Box<dyn BackgroundWorker>) {
        let name = worker.name();
        tracing::info!(worker = name, "Spawning background worker");
        let handle = self.tasks.spawn(worker.run(self.token.clone()));
        self.names.insert(handle.id(), name);
    }

    pub fn spawn_all(&mut self, workers: impl IntoIterator<Item = Box<dyn BackgroundWorker>>) {
        workers.into_iter().for_each(|w| self.spawn(w));
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// First worker to stop, panics included; pends forever while the set is empty.
    pub async fn wait_any(&mut self) -> WorkerExit {
        match self.tasks.join_next_with_id().await {
            Some(joined) => self.exit(joined),
            None => std::future::pending().await,
        }
    }

    /// Cancels every worker and awaits them; stragglers are aborted after `grace`.
    pub async fn shutdown(mut self, grace: Duration) {
        self.token.cancel();
        let drained = tokio::time::timeout(grace, async {
            while let Some(joined) = self.tasks.join_next_with_id().await {
                let exit = self.exit(joined);
                match exit.result {
                    Ok(()) => tracing::info!(worker = exit.name, "Worker stopped"),
                    Err(e) => tracing::error!(worker = exit.name, error = %e, "Worker failed"),
                }
            }
        })
        .await;
        if drained.is_err() {
            tracing::warn!(
                pending = self.tasks.len(),
                "Workers exceeded shutdown grace, aborting"
            );
            self.tasks.abort_all();
        }
    }

    fn exit(&mut self, joined: Result<(Id, Outcome<()>), tokio::task::JoinError>) -> WorkerExit {
        match joined {
            Ok((id, result)) => WorkerExit {
                name: self.names.remove(&id).unwrap_or("unknown"),
                result,
            },
            Err(e) => WorkerExit {
                name: self.names.remove(&e.id()).unwrap_or("unknown"),
                result: Err(Errors::crazy("Worker panicked", Some(Box::new(e)))),
            },
        }
    }
}
