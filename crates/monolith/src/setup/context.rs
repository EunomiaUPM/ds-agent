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

use auth::setup::app::AuthApplication;
use axum::Router;
use catalog_agent::setup::create_root_http_router_with_bus as catalog_http_router_with_bus;
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use negotiation_agent::create_negotiations_http_router_with_bus;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

pub struct CoreContext {
    pub catalog_router: Router,
    pub auth_router: Router,
    pub negotiation_router: Router,
    pub gateway_router: Router,
    pub events_ctx: Arc<events::setup::context::AppContext>,
}

impl CoreContext {
    pub async fn build(config: &ApplicationConfig, vault: Arc<VaultService>) -> Outcome<Self> {
        let config = Arc::new(config.clone());

        // Build events context and event bus
        let events_db = vault.get_db_connection(config.gateway().common()).await?;
        let events_ctx = Arc::new(events::setup::context::AppContext::build(
            events_db.clone(),
            None,
        ));

        // Spawn background retry worker
        events::setup::workers::RetryWorkerHandle::spawn(
            events_ctx.retry_worker.clone(),
            events_ctx.cancel_token.clone(),
        );

        // Gateway context; its validator guards the discovery helpers
        let gateway_validator = oauth::setup::composition::OAuthSetup::new()
            .build_token_service(config.gateway().common().clone().into(), events_db.clone());
        let gateway_ctx = Arc::new(bff::AppContext::new(
            config.gateway().clone(),
            Some(gateway_validator),
        ));

        // Build every free-function agent's HTTP surface once.
        let catalog_router = catalog_http_router_with_bus(
            &config.catalog(),
            vault.clone(),
            Some((*events_ctx.event_bus).clone()),
        )
        .await?;
        let auth_router = AuthApplication::create_router(&config.ssi_auth(), vault.clone()).await?;
        let negotiation_router = create_negotiations_http_router_with_bus(
            &config.contracts(),
            vault.clone(),
            Some((*events_ctx.event_bus).clone()),
        )
        .await;
        let gateway_router = bff::create_gateway_http_router_with_context(gateway_ctx).await;

        Ok(Self {
            catalog_router,
            auth_router,
            negotiation_router,
            gateway_router,
            events_ctx,
        })
    }
}
