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

use crate::modules::GaiaSelfAttesterModule;
use axum::extract::State;
use axum::routing::post;
use axum::Router;
use common::routes::auth::gaia;
use ymir::errors::AppResult;
use ymir::types::oauth::UserInfo;

/// Routes of the Gaia-X self-attestation.
pub struct GaiaSelfAttesterRouter {
    gaia: Arc<dyn GaiaSelfAttesterModule>,
}

impl GaiaSelfAttesterRouter {
    // ==========================================================================================
    // Sub-routers
    // ==========================================================================================

    pub fn new(gaia: Arc<dyn GaiaSelfAttesterModule>) -> Self {
        GaiaSelfAttesterRouter { gaia }
    }

    /// Gaia-X self-attestation; mounted behind the OAuth guard.
    pub fn internal(self) -> Router {
        Router::new()
            .route(gaia::GENERATE, post(Self::generate))
            .with_state(self.gaia)
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (actions)
    // ==========================================================================================

    async fn generate(
        State(gaia): State<Arc<dyn GaiaSelfAttesterModule>>,
        user: UserInfo,
    ) -> AppResult<()> {
        gaia.generate_gaia_vcs(&user).await
    }
}
