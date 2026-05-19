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

pub mod parameters;
pub mod secrets;

use std::sync::Arc;

use axum::Router;

use crate::services::parameters::ParameterStore;
use crate::services::secrets::SecretStore;

pub struct KeystoreRouter {
    parameters: Arc<dyn ParameterStore<serde_json::Value>>,
    secrets: Arc<dyn SecretStore>,
}

impl KeystoreRouter {
    pub fn new(
        parameters: Arc<dyn ParameterStore<serde_json::Value>>,
        secrets: Arc<dyn SecretStore>,
    ) -> Self {
        Self {
            parameters,
            secrets,
        }
    }

    pub fn router(self) -> Router {
        let parameters_router = parameters::ParameterRouter::new(self.parameters).router();
        let secrets_router = secrets::SecretRouter::new(self.secrets).router();

        Router::new()
            .nest("/parameters", parameters_router)
            .nest("/secrets", secrets_router)
    }
}
