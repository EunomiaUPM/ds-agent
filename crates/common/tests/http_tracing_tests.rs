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

//! The HTTP server span continues the caller's W3C trace and names itself by route.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use common::http_tracing::HttpTracing;
use opentelemetry::global;
use opentelemetry::trace::{SpanKind, TracerProvider as _};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};
use tower::ServiceExt;
use tracing_subscriber::layer::SubscriberExt;

const TRACE_ID: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
const PARENT_ID: &str = "00f067aa0ba902b7";

#[tokio::test]
async fn server_span_joins_incoming_traceparent() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = Router::new()
        .route("/items/{id}", get(|| async { StatusCode::NO_CONTENT }))
        .layer(HttpTracing::http_layer());
    let req = Request::get("/items/42")
        .header("traceparent", format!("00-{TRACE_ID}-{PARENT_ID}-01"))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    // The span lives in the response body until it is dropped.
    drop(res);

    provider.force_flush().unwrap();
    let spans = exporter.get_finished_spans().unwrap();
    let server = spans
        .iter()
        .find(|s| s.span_kind == SpanKind::Server)
        .expect("server span exported");
    assert_eq!(server.name, "GET /items/{id}");
    assert_eq!(server.span_context.trace_id().to_string(), TRACE_ID);
    assert_eq!(server.parent_span_id.to_string(), PARENT_ID);
    let route = server
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.route")
        .map(|kv| kv.value.to_string());
    assert_eq!(route.as_deref(), Some("/items/{id}"));
}
