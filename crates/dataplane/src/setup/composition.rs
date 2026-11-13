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

//! Dataplane as a composable module, registered by the transfer agent: control API under
//! `{api}/transfer-agent/dataplane` and the data proxy.

use std::sync::Arc;

use axum::Router;
use common::config::services::traits::TransferConfigTrait;
use common::config::services::TransferConfig;
use ymir::config::traits::CommonConfigTrait;
use ymir::module_loader::root_context::RootContext;
use ymir::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use crate::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::engine::dataplane_manager::dataplane_proxy::HTTP_LISTENER_PATH;
use crate::http::dataplane_info::DataPlaneProcessesRouter;
use crate::http::dataplane_transfer_logs::DataplaneTransferLogsRouter;
use crate::http::transfer_events::TransferEventsRouter;
use crate::proxy::HttpDataProxy;
use crate::relay::KafkaRecoveryWorker;
use crate::setup::context::AppContext;
use crate::setup::ports::DataplanePorts;
use common::routes::dataplane as routes;

/// Dataplane as a module: control API under its prefix and the data proxy.
#[derive(Clone)]
pub struct DataplaneModule {
    prefix: String,
    /// Public base URL of the agent, used to announce the proxy ingress URL to peers.
    public_base_url: String,
    proxy_accept_invalid_certs: bool,
    proxy_allow_private_egress: bool,
    ctx: AppContext,
}

impl DataplaneModule {
    /// Builds the services and the manager on the transfer agent's config.
    pub async fn compose(
        config: &TransferConfig,
        root: &RootContext,
        events: Arc<dyn events::EventSinkTrait>,
        ports: &DataplanePorts,
    ) -> Outcome<Self> {
        Ok(Self {
            prefix: format!(
                "{}/transfer-agent/dataplane",
                config.common().get_api_version()
            ),
            public_base_url: config.common().get_host(HostType::Http),
            proxy_accept_invalid_certs: config.proxy_accept_invalid_certs(),
            proxy_allow_private_egress: config.proxy_allow_private_egress(),
            ctx: AppContext::build(config, root, events, ports).await?,
        })
    }

    /// Manager driven in-process by the transfer agent that hosts the dataplane.
    pub fn local_manager(&self) -> Arc<DataplaneManager> {
        self.ctx.manager.clone()
    }

    /// Guard consulted by the connector service before deleting a connector instance.
    pub fn active_transfer_guard(&self) -> Arc<dyn connector::ActiveTransferGuard> {
        Arc::new(DataplaneActiveTransferGuard {
            repo: self.ctx.repo.clone(),
        })
    }

    /// Static, so the hosting agent's migrator can list them.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::get_dataplane_migrations()
    }

    /// Transfer processes with their logs and events, plus the global event lookup.
    fn control_router(&self) -> Router {
        let ctx = &self.ctx;
        let events_router = TransferEventsRouter::new(ctx.events_svc.clone());
        let processes_router = Router::new()
            .merge(
                DataPlaneProcessesRouter::new(
                    ctx.transfer_svc.clone(),
                    self.public_base_url.clone(),
                )
                .router(),
            )
            .merge(DataplaneTransferLogsRouter::new(ctx.logs_svc.clone()).router())
            .merge(events_router.clone().dataplane_processes_sub_router());
        Router::new()
            .nest(routes::dataplane_processes::PREFIX, processes_router)
            .nest(routes::transfer_events::PREFIX, events_router.events_sub_router())
            .route_layer(axum::middleware::from_fn_with_state(
                ctx.oauth_validator.clone(),
                ymir::http::OauthHttpMiddleware::run,
            ))
    }

    /// Data proxy with keystore-backed lookup and security policy from config.
    fn proxy_router(&self) -> Router {
        let ctx = &self.ctx;
        HttpDataProxy::new(ctx.transfer_svc.clone(), ctx.repo.clone())
            .with_keystore(ctx.keystore_lookup.clone())
            .with_accept_invalid_certs(self.proxy_accept_invalid_certs)
            .with_allow_private_egress(self.proxy_allow_private_egress)
            .with_signals(ctx.signals.clone())
            .with_tracker(ctx.tracker.clone())
            .with_events(ctx.events.clone())
            .with_audit_sender(ctx.audit_tx.clone())
            .router()
    }
}

impl ServiceModuleTrait for DataplaneModule {
    fn name(&self) -> &'static str {
        "dataplane"
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    /// Control API under its prefix; the proxy on the path advertised in data addresses.
    fn http(&self) -> Option<(String, Router)> {
        let router = Router::new()
            .nest(&self.prefix, self.control_router())
            .nest(
                HTTP_LISTENER_PATH.trim_end_matches('/'),
                self.proxy_router(),
            );
        Some((String::new(), router))
    }

    fn workers(&self) -> Vec<Box<dyn ymir::boot::workers::BackgroundWorker>> {
        let ks: Arc<dyn connector::KeystoreLookup> = self.ctx.keystore_lookup.clone();
        let mut workers: Vec<Box<dyn ymir::boot::workers::BackgroundWorker>> =
            vec![Box::new(KafkaRecoveryWorker::new(
                self.ctx.repo.clone(),
                self.ctx.connector.clone(),
                Some(ks),
                self.ctx.kafka_supervisor.clone(),
            ))];
        if let Some(audit) = self.ctx.take_audit_worker() {
            workers.push(Box::new(audit));
        }
        workers
    }
}

struct DataplaneActiveTransferGuard {
    repo: Arc<dyn crate::data::factory_trait::DataplaneRepoTrait>,
}

#[async_trait::async_trait]
impl connector::ActiveTransferGuard for DataplaneActiveTransferGuard {
    async fn count_active_transfers(&self, instance_id: &urn::Urn) -> Outcome<u64> {
        self.repo
            .get_dataplane_transfers_repo()
            .count_active_by_connector_instance(instance_id)
            .await
    }
}
