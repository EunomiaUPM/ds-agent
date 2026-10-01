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

//! End-to-end OTLP export through `Telemetry`. Needs the dev observability stack:
//! `docker compose -f deployment/dev/docker-compose.observability.yaml up -d`, then
//! `cargo test -p common --test otlp_export_tests -- --ignored`.

use std::time::Duration;

use common::telemetry::Telemetry;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

const SERVICE: &str = "otlp-export-smoke";

#[tokio::test]
#[ignore = "needs the dev observability stack"]
async fn spans_reach_jaeger_through_the_collector() {
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");
    let telemetry = Telemetry::init(SERVICE);
    tracing::info_span!("smoke").in_scope(|| tracing::info!("exported span"));
    telemetry.shutdown();

    let url = format!("http://localhost:16686/api/traces?service={SERVICE}&limit=1");
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let found: serde_json::Value = http_client().get_json(&url, None).await.unwrap();
        if found["data"].as_array().is_some_and(|t| !t.is_empty()) {
            return;
        }
    }
    panic!("no trace for {SERVICE} reached Jaeger");
}
