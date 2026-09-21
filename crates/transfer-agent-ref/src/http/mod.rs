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

use axum::Router;
use common::auth::OauthTokenValidator;
use std::sync::Arc;

pub(crate) mod transfer_message_router;
pub(crate) mod transfer_process_router;

pub(crate) struct TransferHttpRouter;

impl TransferHttpRouter {
    pub(crate) fn build(
        process_router: Router,
        message_router: Router,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Router {
        Router::new()
            .merge(process_router)
            .merge(message_router)
            .route_layer(axum::middleware::from_fn_with_state(
                validator,
                common::auth::http::AuthHttpMiddleware::run,
            ))
    }
}
