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

use crate::config::services::TransferConfig;
use std::sync::Arc;

pub fn transfer_config_fixture() -> Arc<TransferConfig> {
    let json = serde_json::json!({
        "common": {
            "hosts": {
                "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null },
                "grpc": null, "graphql": null
            },
            "db": { "db_type": "Postgres", "url": "localhost", "port": "5432" },
            "api": { "version": "v1", "openapi_path": "/openapi.json" },
            "connection": { "is_local": true, "is_prod": false, "is_vault_real": false, "has_tls_proxy": false }
        },
        "cache": { "cache_type": "Noop", "url": "", "port": "", "user": "", "password": "" },
        "contracts": {
            "hosts": { "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null }, "grpc": null, "graphql": null },
            "api_version": "v1"
        },
        "catalog": {
            "hosts": { "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null }, "grpc": null, "graphql": null },
            "api_version": "v1"
        },
        "is_catalog_datahub": false,
        "ssi_auth": {
            "hosts": { "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null }, "grpc": null, "graphql": null },
            "api_version": "v1"
        }
    });
    Arc::new(serde_json::from_value(json).unwrap())
}
