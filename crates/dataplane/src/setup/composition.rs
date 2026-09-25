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
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use ymir::config::traits::ApiConfigTrait;
use ymir::errors::Outcome;

use crate::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::engine::dataplane_manager::dataplane_proxy::HTTP_LISTENER_PATH;
use crate::http::dataplane_info::DataPlaneProcessesRouter;
use crate::http::dataplane_transfer_logs::DataplaneTransferLogsRouter;
use crate::http::transfer_events::TransferEventsRouter;
use crate::setup::context::AppContext;
use crate::setup::ports::DataplanePorts;
use crate::testing_proxy::http::http::TestingHTTPProxy;

#[derive(Clone)]
pub struct DataplaneModule {
    prefix: String,
    ctx: AppContext,
}

impl DataplaneModule {
    pub async fn compose(
        config: &TransferConfig,
        root: &RootContext,
        ports: &DataplanePorts,
    ) -> Outcome<Self> {
        Ok(Self {
            prefix: format!(
                "{}/transfer-agent/dataplane",
                config.common().get_api_version()
            ),
            ctx: AppContext::build(config, root, ports).await?,
        })
    }

    /// Manager driven in-process by the transfer agent that hosts the dataplane.
    pub fn local_manager(&self) -> Arc<DataplaneManager> {
        self.ctx.manager.clone()
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::get_dataplane_migrations()
    }

    /// Transfer processes with their logs and events, plus the global event lookup.
    fn control_router(&self) -> Router {
        let ctx = &self.ctx;
        let events_router = TransferEventsRouter::new(ctx.events_svc.clone());
        let processes_router = Router::new()
            .merge(DataPlaneProcessesRouter::new(ctx.transfer_svc.clone()).router())
            .merge(DataplaneTransferLogsRouter::new(ctx.logs_svc.clone()).router())
            .merge(events_router.clone().dataplane_processes_sub_router());
        Router::new()
            .nest("/dataplane-processes", processes_router)
            .nest("/transfer-events", events_router.events_sub_router())
            .route_layer(axum::middleware::from_fn_with_state(
                ctx.oauth_validator.clone(),
                common::auth::http::AuthHttpMiddleware::run,
            ))
    }

    /// Data proxy with keystore-backed lookup.
    fn proxy_router(&self) -> Router {
        let ctx = &self.ctx;
        TestingHTTPProxy::new(ctx.transfer_svc.clone(), ctx.repo.clone())
            .with_keystore(ctx.keystore_lookup.clone())
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
}
