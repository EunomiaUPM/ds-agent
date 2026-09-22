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

//! Traits implemented by domain event payloads published into the bus.

use serde::Serialize;
use urn::Urn;

use crate::entities::envelope::EventEnvelope;
use crate::entities::topic::Topic;

// Core trait implemented by domain event payloads to be published into the bus.
pub trait Event: Serialize + Send + Sync + 'static {
    // Return the unique event topic name (e.g., "transfers:bla").
    fn event_name() -> &'static str;

    // Return the validated topic instance for this event.
    fn topic() -> Topic {
        Topic::new(Self::event_name()).expect("valid static event topic")
    }

    // Originating crate identifier.
    fn source_crate() -> &'static str {
        "events"
    }

    // Schema version for payload evolution (defaults to 1).
    fn schema_version() -> u32 {
        1
    }

    // Optional correlation identifier for distributed tracing.
    fn correlation_id(&self) -> Option<Urn> {
        None
    }

    // Tenant identifier for tenant isolation. Defaults to "default".
    fn tenant_id(&self) -> &str {
        "default"
    }

    // Convert into an immutable domain envelope.
    fn into_envelope(self) -> EventEnvelope;
}

// Backward-compatible trait for existing event producers.
pub trait IntoEvent: Sized {
    // Return topic identifier for this event.
    fn topic() -> Topic;

    // Return schema version for payload evolution.
    fn schema_version() -> u32 {
        1
    }

    // Return optional correlation identifier.
    fn correlation_id(&self) -> Option<Urn> {
        None
    }

    // Tenant identifier for tenant isolation. Defaults to "default".
    fn tenant_id(&self) -> &str {
        "default"
    }

    // Convert into an event envelope.
    fn into_envelope(self) -> EventEnvelope;
}
