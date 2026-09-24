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

use tokio::signal;

/// Process termination request: Ctrl+C or, on unix, SIGTERM (docker/k8s stop).
pub struct ShutdownSignal;

impl ShutdownSignal {
    pub async fn received() {
        let ctrl_c = async {
            if let Err(e) = signal::ctrl_c().await {
                tracing::error!("Unable to listen for Ctrl+C: {e}");
                std::future::pending::<()>().await;
            }
        };
        #[cfg(unix)]
        let terminate = async {
            match signal::unix::signal(signal::unix::SignalKind::terminate()) {
                Ok(mut sigterm) => {
                    sigterm.recv().await;
                }
                Err(e) => {
                    tracing::error!("Unable to listen for SIGTERM: {e}");
                    std::future::pending::<()>().await;
                }
            }
        };
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }
}
