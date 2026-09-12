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

use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use events::bus::envelope::{EventEnvelope, TopicPattern};
use events::bus::{EventBus, EventBusTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Inbound WebSocket messages sent by browser clients.
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientWsMessage {
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Ping,
}

/// Outbound WebSocket notifications sent to browser clients.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerWsMessage<'a> {
    Event { data: &'a EventEnvelope },
    Subscribed { topic: &'a str },
    Unsubscribed { topic: &'a str },
    Pong,
    Error { message: &'a str },
}

/// Upgrades HTTP connection to WebSocket and streams filtered domain events.
pub struct BffWebSocketHandler {
    event_bus: Option<Arc<EventBus>>,
    legacy_notification_tx: broadcast::Sender<String>,
}

impl BffWebSocketHandler {
    /// Create handler with optional EventBus and legacy fallback channel.
    pub fn new(
        event_bus: Option<Arc<EventBus>>,
        legacy_notification_tx: broadcast::Sender<String>,
    ) -> Self {
        Self {
            event_bus,
            legacy_notification_tx,
        }
    }

    /// Handle Axum WebSocket upgrade request.
    pub async fn upgrade(self: Arc<Self>, ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(move |socket| async move {
            self.handle_socket(socket).await;
        })
    }

    async fn handle_socket(&self, mut socket: WebSocket) {
        debug!("New WebSocket client connected");
        let mut patterns: Vec<TopicPattern> = vec![TopicPattern::match_all()];

        let mut event_rx = self.event_bus.as_ref().map(|bus| bus.subscribe());
        let mut legacy_rx = self.legacy_notification_tx.subscribe();

        loop {
            tokio::select! {
                Some(msg) = async {
                    match event_rx.as_mut() {
                        Some(rx) => rx.recv().await.ok(),
                        None => futures_util::future::pending().await,
                    }
                } => {
                    if Self::matches_any(&patterns, &msg) {
                        let payload = json!({
                            "type": "event",
                            "data": msg
                        }).to_string();

                        if socket.send(Message::Text(Utf8Bytes::from(payload))).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(raw_text) = legacy_rx.recv(), if event_rx.is_none() => {
                    if socket.send(Message::Text(Utf8Bytes::from(raw_text))).await.is_err() {
                        break;
                    }
                }
                Some(inbound) = socket.recv() => {
                    match inbound {
                        Ok(Message::Text(text)) => {
                            self.handle_client_text(&mut socket, &text, &mut patterns).await;
                        }
                        Ok(Message::Ping(p)) => {
                            if socket.send(Message::Pong(p)).await.is_err() {
                                break;
                            }
                        }
                        Ok(Message::Close(_)) | Err(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
                else => break,
            }
        }
        debug!("WebSocket client disconnected");
    }

    async fn handle_client_text(
        &self,
        socket: &mut WebSocket,
        text: &str,
        patterns: &mut Vec<TopicPattern>,
    ) {
        match serde_json::from_str::<ClientWsMessage>(text) {
            Ok(ClientWsMessage::Subscribe { topic }) => match TopicPattern::new(&topic) {
                Ok(pat) => {
                    patterns.push(pat);
                    let resp = json!({ "type": "subscribed", "topic": topic }).to_string();
                    let _ = socket.send(Message::Text(Utf8Bytes::from(resp))).await;
                }
                Err(e) => {
                    let resp =
                        json!({ "type": "error", "message": format!("invalid pattern: {e}") })
                            .to_string();
                    let _ = socket.send(Message::Text(Utf8Bytes::from(resp))).await;
                }
            },
            Ok(ClientWsMessage::Unsubscribe { topic }) => {
                patterns.retain(|p| p.as_str() != topic);
                let resp = json!({ "type": "unsubscribed", "topic": topic }).to_string();
                let _ = socket.send(Message::Text(Utf8Bytes::from(resp))).await;
            }
            Ok(ClientWsMessage::Ping) => {
                let resp = json!({ "type": "pong" }).to_string();
                let _ = socket.send(Message::Text(Utf8Bytes::from(resp))).await;
            }
            Err(_) => {
                let resp = json!({ "type": "error", "message": "unrecognized action" }).to_string();
                let _ = socket.send(Message::Text(Utf8Bytes::from(resp))).await;
            }
        }
    }

    fn matches_any(patterns: &[TopicPattern], envelope: &EventEnvelope) -> bool {
        patterns.iter().any(|p| p.matches(&envelope.topic))
    }
}
