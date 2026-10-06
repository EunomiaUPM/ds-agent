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
use std::time::Duration;

use chrono::Utc;
use common::boot::workers::BackgroundWorker;
use tokio_util::sync::CancellationToken;
use tracing::info;
use ymir::errors::Outcome;

use crate::modules::{GateKeeperModule, PeerConnectorModule};
use crate::types::token_lifetimes::SWEEP_INTERVAL_SECS;

pub struct GrantExpiryWorker {
    gatekeeper: Arc<dyn GateKeeperModule>,
    peer_connector: Arc<dyn PeerConnectorModule>,
}

impl GrantExpiryWorker {
    pub fn new(
        gatekeeper: Arc<dyn GateKeeperModule>,
        peer_connector: Arc<dyn PeerConnectorModule>,
    ) -> Self {
        Self {
            gatekeeper,
            peer_connector,
        }
    }

    async fn sweep(&self) {
        let now = Utc::now();
        match self.gatekeeper.sweep_expired(now).await {
            Ok(0) => {}
            Ok(count) => info!(count, "Finalized expired received grants"),
            Err(e) => e.log(),
        }
        match self.peer_connector.sweep_expired(now).await {
            Ok(0) => {}
            Ok(count) => info!(count, "Finalized expired sent grants"),
            Err(e) => e.log(),
        }
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for GrantExpiryWorker {
    fn name(&self) -> &'static str {
        "grant-expiry"
    }

    async fn run(self: Box<Self>, cancel_token: CancellationToken) -> Outcome<()> {
        let mut ticker = tokio::time::interval(Duration::from_secs(SWEEP_INTERVAL_SECS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = ticker.tick() => self.sweep().await,
            }
        }
        Ok(())
    }
}
