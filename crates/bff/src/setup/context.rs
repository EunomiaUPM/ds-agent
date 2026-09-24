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

use crate::proxy::HttpProxyDispatcher;

/// Configuration, reverse proxy and, when a database is available, the OAuth validator.
#[derive(Clone)]
pub struct AppContext {
    pub config: GatewayConfig,
    pub oauth_validator: Option<Arc<dyn OauthTokenValidator>>,
    pub proxy: Arc<HttpProxyDispatcher>,
}

impl AppContext {
    pub fn new(
        config: GatewayConfig,
        oauth_validator: Option<Arc<dyn OauthTokenValidator>>,
    ) -> Self {
        let proxy = Arc::new(HttpProxyDispatcher::new(config.clone()));
        Self {
            config,
            oauth_validator,
            proxy,
        }
    }
}
