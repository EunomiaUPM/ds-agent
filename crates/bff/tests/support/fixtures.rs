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

//! Gateway configuration and a local HTTP server to proxy to.

use axum::Router;
use common::config::services::GatewayConfig;
use serde_json::json;
use tokio::net::TcpListener;

/// Gateway config whose four upstream agents all point at `127.0.0.1:upstream_port`.
pub fn gateway_config(upstream_port: u16) -> GatewayConfig {
    let p_str = upstream_port.to_string();
    let upstream = || {
        json!({
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": p_str, "internal_port": p_str },
                "grpc": null,
                "graphql": null
            },
            "api_version": "v1"
        })
    };
    let raw = json!({
        "common": {
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": "8080", "internal_port": "8080" },
                "grpc": null,
                "graphql": null
            },
            "db": { "db_type": "Postgres", "url": "localhost", "port": "5432" },
            "api": { "version": "v1", "openapi_path": "/openapi.json" },
            "connection": { "is_local": true, "is_prod": false, "is_vault_real": false, "has_tls_proxy": false }
        },
        "is_production": false,
        "is_catalog_datahub": false,
        "catalog": upstream(),
        "contracts": upstream(),
        "transfer": upstream(),
        "ssi_auth": upstream()
    });
    serde_json::from_value(raw).expect("valid config")
}

/// Serves `router` on a random local port and returns that port.
pub async fn serve(router: Router) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    port
}
