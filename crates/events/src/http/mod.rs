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

pub mod dlq;
pub mod events;
pub mod subscriptions;

use axum::Router;
use std::sync::Arc;

pub use dlq::DeadLetterRouter;
pub use events::EventsRouter;
pub use subscriptions::SubscriptionsRouter;

use crate::services::event_bus::EventBus;
use common::auth::http::AuthHttpMiddleware;
use common::auth::OauthTokenValidator;

/// Events API: the feed and its live stream at the root, plus subscriptions and the DLQ.
pub struct EventsHttpRouter;

impl EventsHttpRouter {
    pub fn build(bus: Arc<EventBus>, validator: Arc<dyn OauthTokenValidator>) -> Router {
        Router::new()
            .merge(EventsRouter::new(bus.clone()).router())
            .nest(
                "/subscriptions",
                SubscriptionsRouter::new(bus.subscription_repo()).router(),
            )
            .nest("/dlq", DeadLetterRouter::new(bus).router())
            .route_layer(axum::middleware::from_fn_with_state(
                validator,
                AuthHttpMiddleware::run,
            ))
    }
}
