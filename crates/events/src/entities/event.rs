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

use common::oauth::Owner;
use serde::Serialize;
use urn::Urn;

use crate::entities::envelope::EventEnvelope;
use crate::entities::topic::Topic;

/// Typed domain event; usually implemented with the `event!` macro.
pub trait Event: Serialize + Send + Sync + 'static {
    /// Topic name, e.g. `transfers:started`.
    fn event_name() -> &'static str;

    fn topic() -> Topic {
        Topic::new(Self::event_name()).expect("valid static event topic")
    }

    /// Crate that publishes the event; defaults to `events`.
    fn source_crate() -> &'static str {
        "events"
    }

    /// Schema version for payload evolution (defaults to 1).
    fn schema_version() -> u32 {
        1
    }

    /// `None` unless the event belongs to a wider flow.
    fn correlation_id(&self) -> Option<Urn> {
        None
    }

    /// Owner of the record the event is about.
    fn owner(&self) -> &Owner;

    fn into_envelope(self) -> EventEnvelope;
}

/// Older form of `Event`, kept for existing producers.
pub trait IntoEvent: Sized {
    fn topic() -> Topic;

    /// Defaults to 1.
    fn schema_version() -> u32 {
        1
    }

    fn correlation_id(&self) -> Option<Urn> {
        None
    }

    /// Owner of the record the event is about.
    fn owner(&self) -> &Owner;

    fn into_envelope(self) -> EventEnvelope;
}
