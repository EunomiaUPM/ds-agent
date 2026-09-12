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

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::response::sse::{Event, KeepAlive, Sse};
use events::bus::envelope::TopicPattern;
use events::bus::{EventBus, EventBusTrait};
use futures_util::stream::{self, Stream};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub topic: Option<String>,
}

pub struct SseStreamHandler;

impl SseStreamHandler {
    /// Stream real-time domain events over Server-Sent Events (SSE).
    pub fn stream_events(
        bus: Arc<EventBus>,
        query: SseQuery,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let pattern = query
            .topic
            .and_then(|t| TopicPattern::new(t).ok())
            .unwrap_or_else(TopicPattern::match_all);

        let rx = bus.subscribe();
        let stream = stream::unfold((rx, pattern), |(mut rx, pattern)| async move {
            loop {
                match rx.recv().await {
                    Ok(envelope) if pattern.matches(&envelope.topic) => {
                        if let Ok(data) = serde_json::to_string(&envelope) {
                            let event = Event::default().event(envelope.topic.as_str()).data(data);
                            return Some((Ok(event), (rx, pattern)));
                        }
                    }
                    Ok(_) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                }
            }
        });

        Sse::new(stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
    }
}
