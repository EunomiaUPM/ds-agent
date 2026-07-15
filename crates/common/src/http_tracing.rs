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

use axum::extract::Request;
use axum::response::IntoResponse;
use axum::{serve, Router};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;

/// Per-request tracing span keyed by `x-request-id` (generated if absent).
pub fn trace_layer() -> TraceLayer<
    tower_http::trace::HttpMakeClassifier,
    impl Fn(&Request<axum::body::Body>) -> tracing::Span + Clone,
    impl Fn(&Request<axum::body::Body>, &tracing::Span) + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|req: &Request<_>| {
            let request_id = req
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            tracing::info_span!("request", id = %request_id)
        })
        .on_request(|req: &Request<_>, _span: &tracing::Span| {
            tracing::info!("{} {}", req.method(), req.uri());
        })
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
}
