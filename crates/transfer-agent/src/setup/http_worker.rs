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

use axum::{Router, serve};
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::http_global_404::global_handler_404;
use common::http_tracing::trace_layer;
use common::well_known::WellKnownRoot;
use common::worker_utils::{bind_listener, shutdown_signal, spawn_server};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

pub struct TransferHttpWorker {}

impl TransferHttpWorker {
    pub async fn spawn(
        config: &TransferConfig,
        api_router: Router,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let well_known = WellKnownRoot::get_well_known_router(&config.into())?;
        let router = api_router
            .merge(well_known)
            .fallback(global_handler_404)
            .layer(trace_layer());

        let listener =
            bind_listener(config.common().get_internal_port(HostType::Http), "HTTP").await?;

        let server =
            serve(listener, router).with_graceful_shutdown(shutdown_signal(token.clone(), "HTTP"));

        Ok(spawn_server("HTTP", server))
    }
}
