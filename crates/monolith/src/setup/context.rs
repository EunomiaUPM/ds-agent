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
use bff::create_gateway_http_router;
use catalog_agent::setup::create_root_http_router as catalog_http_router;
use common::config::ApplicationConfig;
use negotiation_agent::create_negotiations_http_router;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;


pub struct CoreContext {
    pub catalog_router: Router,
    pub auth_router: Router,
    pub negotiation_router: Router,
    pub gateway_router: Router,
}

impl CoreContext {
    pub async fn build(config: &ApplicationConfig, vault: Arc<VaultService>) -> Outcome<Self> {
        let config = Arc::new(config.clone());

        // Build every free-function agent's HTTP surface once.
        let catalog_router = catalog_http_router(&config.catalog(), vault.clone()).await?;
        let auth_router = AuthApplication::create_router(&config.ssi_auth(), vault.clone()).await?;
        let negotiation_router =
            create_negotiations_http_router(&config.contracts(), vault.clone()).await;
        let gateway_router = create_gateway_http_router(&config.gateway()).await;

        Ok(Self {
            catalog_router,
            auth_router,
            negotiation_router,
            gateway_router,
        })
    }
}
