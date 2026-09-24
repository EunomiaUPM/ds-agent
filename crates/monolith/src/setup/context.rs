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

use auth::setup::AuthModule;
use axum::Router;
use bff::BffModule;
use catalog_agent::setup::create_root_http_router as create_catalog_http_router;
use common::config::ApplicationConfig;
use common::module_loader::root_context::RootContext;
use negotiation_agent::create_negotiations_http_router;
use ymir::errors::Outcome;

pub struct CoreContext {
    pub catalog_router: Router,
    pub negotiation_router: Router,
    pub auth: AuthModule,
    pub gateway: BffModule,
    pub events_ctx: Arc<events::setup::context::AppContext>,
}

impl CoreContext {
    pub async fn build(config: &ApplicationConfig, root: &RootContext) -> Outcome<Self> {
        let events_ctx = Arc::new(events::setup::context::AppContext::build(
            root.db.clone(),
            None,
        ));
        let bus = (*events_ctx.event_bus).clone();

        // Gateway validator guards the discovery helpers
        let gateway_ctx = Arc::new(bff::AppContext::new(
            config.gateway().clone(),
            Some(root.validator.clone()),
        ));

        // Agents still exposed as free functions build their HTTP surface once.
        let catalog_router =
            create_catalog_http_router(config.catalog(), root, Some(bus.clone())).await?;
        let auth = AuthModule::compose(config.ssi_auth(), root).await?;
        let negotiation_router =
            create_negotiations_http_router(config.contracts(), root, Some(bus)).await;
        let gateway = BffModule::new(gateway_ctx);

        Ok(Self {
            catalog_router,
            negotiation_router,
            auth,
            gateway,
            events_ctx,
        })
    }
}
