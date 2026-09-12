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

pub mod dlq_router {
    pub use super::dlq::*;
}
pub mod events_router {
    pub use super::events::*;
}
pub mod subscriptions_router {
    pub use super::subscriptions::*;
}

use crate::services::event_bus::EventBus;

// Combine all event bus HTTP sub-routers into a unified root router.
pub struct EventsHttpRouter;

impl EventsHttpRouter {
    // Construct the unified events HTTP router nesting events, subscriptions, and DLQ.
    pub fn build(bus: Arc<EventBus>) -> Router {
        let events_subrouter = EventsRouter::new(bus.clone()).router();
        Router::new()
            .merge(events_subrouter.clone())
            .nest("/events", events_subrouter)
            .nest(
                "/subscriptions",
                SubscriptionsRouter::new(bus.subscription_repo()).router(),
            )
            .nest("/dlq", DeadLetterRouter::new(bus).router())
    }
}
