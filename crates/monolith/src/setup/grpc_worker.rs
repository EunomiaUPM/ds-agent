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

use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use common::module_loader::service_composer::ServiceComposer;
use common::worker_utils::GrpcServer;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

pub struct CoreGrpcWorker;

impl CoreGrpcWorker {
    /// Serves every hosted agent's gRPC plane on the monolith's gRPC host; `None` when
    /// no gRPC host is configured.
    pub async fn spawn(
        config: &ApplicationConfig,
        composer: &ServiceComposer,
        token: &CancellationToken,
    ) -> Outcome<Option<JoinHandle<()>>> {
        let common = config.monolith().common();
        if common.grpc().is_none() {
            tracing::warn!("No gRPC host configured, skipping gRPC subsystem");
            return Ok(None);
        }
        tracing::info!(
            "Starting Dataspace gRPC server in {}",
            common.get_host(HostType::Grpc)
        );
        let handle = GrpcServer::spawn(
            common.get_internal_port(HostType::Grpc),
            composer.grpc_routes(),
            composer.grpc_descriptors(),
            token,
        )
        .await?;
        Ok(Some(handle))
    }
}
