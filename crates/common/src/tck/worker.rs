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

//! Plays queued scenario steps through the agent's outbound flow.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use ymir::errors::Outcome;

use crate::flow::FlowError;
use crate::tck::{TckJob, TckJobs, TckStep};
use ymir::boot::workers::BackgroundWorker;

/// Sends one scenario message on the process with pid `pid`, usually through a flow.
#[async_trait::async_trait]
pub trait TckPlayerTrait<M>: Send + Sync {
    async fn send(&self, pid: &str, message: M) -> Result<(), FlowError>;
}

/// Receives jobs and plays each in its own task, so scenarios can overlap.
pub struct TckWorker<M: 'static> {
    name: &'static str,
    jobs: TckJobs<M>,
    player: Arc<dyn TckPlayerTrait<M>>,
}

impl<M: Copy + Debug + Send + Sync + 'static> TckWorker<M> {
    /// Gap the TCK mocks leave between two actions, and before the first.
    const PAUSE: Duration = Duration::from_millis(1000);
    const ATTEMPTS: u64 = 5;

    pub fn new(name: &'static str, jobs: TckJobs<M>, player: Arc<dyn TckPlayerTrait<M>>) -> Self {
        Self { name, jobs, player }
    }

    async fn play(player: Arc<dyn TckPlayerTrait<M>>, job: TckJob<M>) {
        tokio::time::sleep(Self::PAUSE).await;
        for step in job.steps {
            match step {
                TckStep::Pause => tokio::time::sleep(Self::PAUSE).await,
                TckStep::Send(message) => Self::send(player.as_ref(), &job.pid, *message).await,
            }
        }
    }

    /// Sends one message, retrying while the peer is not ready for it yet.
    async fn send(player: &dyn TckPlayerTrait<M>, pid: &str, message: M) {
        for attempt in 1..=Self::ATTEMPTS {
            match player.send(pid, message).await {
                Ok(()) => return,
                Err(err) => {
                    tracing::warn!(pid, ?message, attempt, error = %err, "TCK scenario step failed");
                    tokio::time::sleep(Duration::from_millis(400 * attempt)).await;
                }
            }
        }
        tracing::error!(pid, ?message, "TCK scenario step gave up");
    }
}

#[async_trait::async_trait]
impl<M: Copy + Debug + Send + Sync + 'static> BackgroundWorker for TckWorker<M> {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(mut self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        let mut scenarios = JoinSet::new();
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                job = self.jobs.0.recv() => match job {
                    Some(job) => {
                        scenarios.spawn(Self::play(self.player.clone(), job));
                    }
                    None => token.cancelled().await,
                },
                Some(_) = scenarios.join_next(), if !scenarios.is_empty() => {}
            }
        }
        scenarios.shutdown().await;
        Ok(())
    }
}
