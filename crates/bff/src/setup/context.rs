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

use common::auth::OauthTokenValidator;
use common::config::services::GatewayConfig;
use events::bus::EventBus;
use tokio::sync::broadcast;

use crate::proxy::HttpProxyDispatcher;

/// Shared application context holding configuration, event bus, OAuth validator, and proxy.
#[derive(Clone)]
pub struct AppContext {
    pub config: GatewayConfig,
    pub event_bus: Option<Arc<EventBus>>,
    pub oauth_validator: Option<Arc<dyn OauthTokenValidator>>,
    pub proxy: Arc<HttpProxyDispatcher>,
    pub legacy_notification_tx: broadcast::Sender<String>,
}

impl AppContext {
    /// Initialize application context with optional event bus and OAuth validator.
    pub fn new(
        config: GatewayConfig,
        event_bus: Option<Arc<EventBus>>,
        oauth_validator: Option<Arc<dyn OauthTokenValidator>>,
    ) -> Self {
        let proxy = Arc::new(HttpProxyDispatcher::new(config.clone()));
        let (legacy_notification_tx, _) = broadcast::channel(100);

        Self {
            config,
            event_bus,
            oauth_validator,
            proxy,
            legacy_notification_tx,
        }
    }
}
