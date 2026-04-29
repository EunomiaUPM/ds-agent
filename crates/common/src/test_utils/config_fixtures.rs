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
