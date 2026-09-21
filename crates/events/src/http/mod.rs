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
        Self::build_internal(bus, None)
    }

    // Construct the unified events HTTP router with AuthHttpMiddleware applied.
    pub fn build_with_validator(
        bus: Arc<EventBus>,
        validator: Arc<dyn common::auth::OauthTokenValidator>,
    ) -> Router {
        Self::build_internal(bus, Some(validator))
    }

    fn build_internal(
        bus: Arc<EventBus>,
        validator: Option<Arc<dyn common::auth::OauthTokenValidator>>,
    ) -> Router {
        let events_subrouter = EventsRouter::new(bus.clone()).router();
        let mut router = Router::new()
            .merge(events_subrouter.clone())
            .nest("/events", events_subrouter)
            .nest(
                "/subscriptions",
                SubscriptionsRouter::new(bus.subscription_repo()).router(),
            )
            .nest("/dlq", DeadLetterRouter::new(bus).router());

        if let Some(val) = validator {
            router = router.route_layer(axum::middleware::from_fn_with_state(
                val,
                common::auth::http::AuthHttpMiddleware::run,
            ));
        } else {
            router = router.route_layer(axum::middleware::from_fn(
                |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
                    if req.extensions().get::<common::auth::Claims>().is_none() {
                        let default_claims = common::auth::Claims {
                            sub: "default".to_string(),
                            role: common::auth::RbacRole::Admin,
                            iat: 0,
                            exp: i64::MAX,
                        };
                        req.extensions_mut().insert(default_claims);
                    }
                    next.run(req).await
                },
            ));
        }

        router
    }
}
