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

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use urn::Urn;
use uuid::Uuid;

use crate::entities::commands::PublishEventRequest;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::EventEnvelope;
use crate::entities::queries::ListEventsQuery;
use crate::entities::topic::Topic;
use crate::errors::EventBusError;
use crate::services::event_bus::EventBus;

// Axum HTTP router handling event publishing, listing, and delivery tracking.
#[derive(Clone)]
pub struct EventsRouter {
    bus: Arc<EventBus>,
}

impl EventsRouter {
    // Create router with shared EventBus reference.
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    // Build Axum router mounting all event operations.
    pub fn router(self) -> Router {
        Router::new()
            .route("/publish", post(Self::handle_publish))
            .route("/", get(Self::handle_list_events))
            .route("/{id}", get(Self::handle_get_event))
            .route("/{id}/deliveries", get(Self::handle_get_deliveries))
            .with_state(self.bus)
    }

    // Handler to publish a domain event via HTTP.
    async fn handle_publish(
        State(bus): State<Arc<EventBus>>,
        Json(req): Json<PublishEventRequest>,
    ) -> Result<(StatusCode, Json<EventEnvelope>), EventBusError> {
        let topic = Topic::new(req.topic).map_err(EventBusError::InvalidTopic)?;
        let correlation_id = match req.correlation_id {
            Some(ref s) => {
                Some(Urn::from_str(s).map_err(|e| EventBusError::InvalidUrn(e.to_string()))?)
            }
            None => None,
        };

        let envelope = EventEnvelope::new(
            topic,
            req.source_crate.unwrap_or_else(|| "events".to_string()),
            req.schema_version.unwrap_or(1),
            correlation_id,
            req.payload,
        );

        let published = bus.publish(envelope).await?;
        Ok((StatusCode::CREATED, Json(published)))
    }

    // Handler to query and list stored event envelopes with optional topic filtering.
    async fn handle_list_events(
        State(bus): State<Arc<EventBus>>,
        Query(query): Query<ListEventsQuery>,
    ) -> Result<Json<Vec<EventEnvelope>>, EventBusError> {
        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);
        let events = bus
            .event_repo()
            .list_events(query.topic.as_deref(), limit, offset)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        Ok(Json(events))
    }

    // Handler to fetch a single event by its URN or raw UUID.
    async fn handle_get_event(
        State(bus): State<Arc<EventBus>>,
        Path(id): Path<String>,
    ) -> Result<Json<EventEnvelope>, EventBusError> {
        let (urn, uuid) = Self::parse_id(&id)?;

        let event = bus
            .event_repo()
            .get_event_by_id(&urn)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?
            .ok_or(EventBusError::EventNotFound(uuid))?;

        Ok(Json(event))
    }

    // Validate and parse an ID string into a canonical URN and UUID.
    fn parse_id(id: &str) -> Result<(Urn, Uuid), EventBusError> {
        if let Some(uuid_str) = id.strip_prefix("urn:uuid:") {
            let u =
                Uuid::parse_str(uuid_str).map_err(|e| EventBusError::InvalidUrn(e.to_string()))?;
            let urn = Urn::from_str(id).map_err(|e| EventBusError::InvalidUrn(e.to_string()))?;
            Ok((urn, u))
        } else if let Ok(u) = Uuid::parse_str(id) {
            let urn = Urn::from_str(&format!("urn:uuid:{u}"))
                .map_err(|e| EventBusError::InvalidUrn(e.to_string()))?;
            Ok((urn, u))
        } else {
            Err(EventBusError::InvalidUrn(
                "id must be a valid UUID or urn:uuid:<uuid>".to_string(),
            ))
        }
    }

    // Handler to list delivery history records for an event.
    async fn handle_get_deliveries(
        State(bus): State<Arc<EventBus>>,
        Path(id): Path<String>,
    ) -> Result<Json<Vec<EventDeliveryRecord>>, EventBusError> {
        let deliveries = bus
            .delivery_repo()
            .list_by_event(&id)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        Ok(Json(deliveries))
    }
}
