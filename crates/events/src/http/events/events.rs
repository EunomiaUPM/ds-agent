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

//! Event routes.

use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use axum::{Json, Router};
use common::oauth::{Owner, OwnerScope, UserInfo};
use common::paginated_spec::{Cursor, Paginated};
use common::query::{QueryFilter, QuerySpec};
use futures_util::stream::{self, Stream};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::{AppResult, Errors};

use crate::entities::commands::PublishEventRequest;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::EventEnvelope;
use crate::entities::queries::EventFilter;
use crate::entities::topic::Topic;
use crate::entities::topic_pattern::TopicPattern;
use crate::services::event_bus::EventBus;

/// Query string of the event listing.
pub type EventsQuery = QuerySpec<EventFilter>;

/// Query string of the live stream: a topic pattern.
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    #[serde(flatten)]
    pub filter: EventFilter,
}

/// Event routes: publish, list, live stream and the deliveries of each event.
#[derive(Clone)]
pub struct EventsRouter {
    bus: Arc<EventBus>,
}

impl EventsRouter {
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/",
                get(Self::handle_list_events).post(Self::handle_publish),
            )
            .route("/stream", get(Self::handle_stream))
            .route("/{id}", get(Self::handle_get_event))
            .route("/{id}/deliveries", get(Self::handle_get_deliveries))
            .with_state(self.bus)
    }

    async fn handle_publish(
        State(bus): State<Arc<EventBus>>,
        user: UserInfo,
        Json(req): Json<PublishEventRequest>,
    ) -> AppResult<(StatusCode, Json<EventEnvelope>)> {
        let owner = Owner::of(&user, req.visibility.clone());
        let topic = Topic::new(req.topic).map_err(|e| Errors::validation(e, None))?;
        let correlation_id = match req.correlation_id {
            Some(ref s) => {
                Some(Urn::from_str(s).map_err(|e| Errors::validation(e.to_string(), None))?)
            }
            None => None,
        };

        let envelope = EventEnvelope::new(
            owner,
            topic,
            req.source_crate.unwrap_or_else(|| "events".to_string()),
            req.schema_version.unwrap_or(1),
            correlation_id,
            req.payload,
        );

        let published = bus.publish(envelope).await?;
        Ok((StatusCode::CREATED, Json(published)))
    }

    async fn handle_list_events(
        State(bus): State<Arc<EventBus>>,
        user: UserInfo,
        Query(query): Query<EventsQuery>,
    ) -> AppResult<Json<Paginated<EventEnvelope>>> {
        query.filter.validate()?;
        let page = query.page.clamped();
        let (events, total) = bus
            .event_repo()
            .list_events(
                &OwnerScope::seeing(&user),
                &query.filter,
                &page,
                &query.sort,
            )
            .await?;
        Ok(Json(Paginated::from_page(
            events,
            &page,
            Some(total),
            |last| Cursor::encode_composite(&last.timestamp, last.id.as_ref()),
        )))
    }

    /// Server-sent events the caller sees, optionally narrowed by `topic`.
    async fn handle_stream(
        State(bus): State<Arc<EventBus>>,
        user: UserInfo,
        Query(query): Query<StreamQuery>,
    ) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
        let scope = OwnerScope::seeing(&user);
        let pattern = query
            .filter
            .topic_pattern()?
            .unwrap_or_else(TopicPattern::match_all);
        let receiver = bus.subscribe();
        let events = stream::unfold(
            (receiver, pattern, scope),
            |(mut receiver, pattern, scope)| async move {
                loop {
                    match receiver.recv().await {
                        Ok(envelope)
                            if scope.admits_owner(&envelope.owner)
                                && pattern.matches(&envelope.topic) =>
                        {
                            let Ok(data) = serde_json::to_string(&envelope) else {
                                continue;
                            };
                            // Unnamed events, so browsers receive them through `onmessage`.
                            let event = Event::default().data(data);
                            return Some((Ok(event), (receiver, pattern, scope)));
                        }
                        Ok(_) | Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => return None,
                    }
                }
            },
        );
        Ok(Sse::new(events).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
    }

    async fn handle_get_event(
        State(bus): State<Arc<EventBus>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<EventEnvelope>> {
        let (urn, uuid) = Self::parse_id(&id)?;
        let event = bus
            .event_repo()
            .get_event_by_id(&OwnerScope::seeing(&user), &urn)
            .await?
            .ok_or_else(|| Errors::missing_resource(uuid.to_string(), "event not found", None))?;
        Ok(Json(event))
    }

    async fn handle_get_deliveries(
        State(bus): State<Arc<EventBus>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Vec<EventDeliveryRecord>>> {
        let deliveries = bus
            .delivery_repo()
            .list_by_event(&OwnerScope::seeing(&user), &id)
            .await?;
        Ok(Json(deliveries))
    }

    // Validate and parse an ID string into a canonical URN and UUID.
    fn parse_id(id: &str) -> AppResult<(Urn, Uuid)> {
        if let Some(uuid_str) = id.strip_prefix("urn:uuid:") {
            let u =
                Uuid::parse_str(uuid_str).map_err(|e| Errors::validation(e.to_string(), None))?;
            let urn = Urn::from_str(id).map_err(|e| Errors::validation(e.to_string(), None))?;
            Ok((urn, u))
        } else if let Ok(u) = Uuid::parse_str(id) {
            let urn = Urn::from_str(&format!("urn:uuid:{u}"))
                .map_err(|e| Errors::validation(e.to_string(), None))?;
            Ok((urn, u))
        } else {
            Err(Errors::validation(
                "id must be a valid UUID or urn:uuid:<uuid>",
                None,
            ))
        }
    }
}
