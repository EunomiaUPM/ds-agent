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

//! One participant: events, catalog and negotiation composed as the monolith does, over its own
//! database and served on a local port, with a stub validator for the management API.

use std::sync::{Arc, Once};

use catalog_agent::setup::{CatalogAgentModule, CatalogPorts};
use common::boot::seeders::BootPhase;
use common::boot::workers::WorkerSet;
use common::config::services::{CommonConfig, ContractsConfig, TransferConfig};
use common::config::types::min_known_config::MinKnownConfig;
use common::facades::AuthPorts;
use common::module_loader::root_context::RootContext;
use common::module_loader::service_composer::ServiceComposer;
use common::well_known::WellKnownRoot;
use events::setup::EventsModule;
use negotiation_agent::setup::{NegotiationAgentModule, NegotiationPorts};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use transfer_agent::setup::{TransferAgentModule, TransferPorts};
use ymir::services::client::ClientTrait;
use ymir::types::http::HttpBody;
use ymir::services::client::ClientExt;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::utils::{bearer_headers, http_client};

use super::auth_stub::OwnerValidator;
use super::database::TestDatabase;

/// A running participant; it stops serving when dropped.
pub struct Participant {
    pub did: String,
    /// Unique per run, so participants and runs never share cache keys.
    pub tenant: String,
    pub base_url: String,
    shutdown: CancellationToken,
    _db: TestDatabase,
}

impl Participant {
    /// Composes and serves a participant of `tenant` on `listener`, seeded like a fresh
    /// deployment.
    pub async fn start(did: &str, tenant: &str, listener: TcpListener, auth: AuthPorts) -> Self {
        Self::init_tracing();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{port}");
        let common = Self::common_config(port);
        let db = TestDatabase::create().await;
        let root = RootContext {
            vault: Arc::new(Self::vault()),
            db: db.connection.clone(),
            validator: Arc::new(OwnerValidator {
                tenant: tenant.to_string(),
            }),
        };

        let events = EventsModule::compose(&root);
        let bus = Some(events.event_bus());
        let catalog = CatalogAgentModule::compose(
            &Self::section(
                &common,
                json!({
                    "cache": Self::cache_config(),
                    "policy_templates_folder": null,
                    "datahub": null,
                    "ssi_auth": Self::known(port),
                    "contracts": Self::known(port),
                }),
            ),
            &root,
            bus.clone(),
            &CatalogPorts::local(auth.clone()),
        )
        .await
        .expect("compose catalog");
        let negotiation = NegotiationAgentModule::compose(
            &Self::section::<ContractsConfig>(
                &common,
                json!({
                    "ssi_auth": Self::known(port),
                    "catalog": Self::known(port),
                    "is_catalog_datahub": false,
                }),
            ),
            &root,
            bus.clone(),
            &NegotiationPorts::local(auth.clone(), catalog.catalog_facade()),
        )
        .await
        .expect("compose negotiation");
        // Transfer brings the dataplane, which needs Redis: only composed when REDIS_URL is set.
        let transfer = match std::env::var("REDIS_URL") {
            Ok(_) => {
                let config = Self::section::<TransferConfig>(
                    &common,
                    json!({
                        "cache": Self::cache_config(),
                        "contracts": Self::known(port),
                        "catalog": Self::known(port),
                        "is_catalog_datahub": false,
                        "ssi_auth": Self::known(port),
                    }),
                );
                let ports = TransferPorts::local(
                    &config,
                    &root,
                    auth.clone(),
                    negotiation.negotiation_facade(),
                    catalog.catalog_facade(),
                    catalog.local_connector_instances(),
                )
                .await
                .expect("compose transfer ports");
                Some(TransferAgentModule::compose(&config, &root, bus, &ports))
            }
            Err(_) => None,
        };
        let composer = ServiceComposer::new()
            .register(events)
            .register(catalog)
            .register(negotiation);
        let composer = match transfer {
            Some(transfer) => composer.register(transfer),
            None => composer,
        };

        let (before, after): (Vec<_>, Vec<_>) = composer
            .seeders()
            .into_iter()
            .partition(|s| s.phase() == BootPhase::BeforeServe);
        for seeder in before {
            seeder.seed().await.expect("seed before serving");
        }
        let router = composer.http_router().merge(
            WellKnownRoot::get_well_known_router(
                &MinKnownConfig::from(&common),
                Some(auth.mates.clone()),
            )
            .expect("well-known router"),
        );
        let shutdown = CancellationToken::new();
        let stop = shutdown.clone();
        tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(stop.cancelled_owned())
                .await
                .unwrap();
        });
        let mut workers = WorkerSet::new(shutdown.clone());
        workers.spawn_all(composer.workers());
        tokio::spawn(async move {
            workers.wait_any().await;
        });
        for seeder in after {
            seeder.seed().await.expect("seed after serving");
        }

        Self {
            did: did.to_string(),
            tenant: tenant.to_string(),
            base_url,
            shutdown,
            _db: db,
        }
    }

    /// Logs to stderr when `RUST_LOG` is set, to see where a participant gets stuck.
    fn init_tracing() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            if std::env::var("RUST_LOG").is_ok() {
                let _ = tracing_subscriber::fmt()
                    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                    .with_test_writer()
                    .try_init();
            }
        });
    }

    /// GET `path` as the owner of the participant's tenant.
    pub async fn get(&self, path: &str) -> Value {
        http_client()
            .get_json(&self.url(path), Some(bearer_headers("owner").unwrap()))
            .await
            .unwrap_or_else(|e| panic!("GET {path}: {e:?}"))
    }

    /// POST `body` to `path` as the owner of the participant's tenant.
    pub async fn post(&self, path: &str, body: Value) -> Value {
        http_client()
            .post_json(
                &self.url(path),
                Some(bearer_headers("owner").unwrap()),
                &body,
            )
            .await
            .unwrap_or_else(|e| panic!("POST {path}: {e:?}"))
    }

    /// POST `body` to `path` as the owner, with extra `headers`.
    pub async fn post_with(&self, path: &str, body: Value, headers: &[(&str, &str)]) -> Value {
        let mut map = bearer_headers("owner").unwrap();
        for (name, value) in headers {
            map.insert(
                axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                axum::http::HeaderValue::from_str(value).unwrap(),
            );
        }
        http_client()
            .post_json(&self.url(path), Some(map), &body)
            .await
            .unwrap_or_else(|e| panic!("POST {path}: {e:?}"))
    }

    /// POST a DSP message to `path` as a peer presenting `token`; answers status and body.
    pub async fn post_dsp(&self, path: &str, token: &str, body: &Value) -> (u16, Value) {
        let response = http_client()
            .post(&self.url(path), Some(bearer_headers(token).unwrap()), HttpBody::Json(body.clone()))
            .await
            .unwrap_or_else(|e| panic!("POST {path}: {e:?}"));
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    /// GET `path` as a peer presenting `token`; answers status and body.
    pub async fn get_dsp(&self, path: &str, token: &str) -> (u16, Value) {
        let response = http_client()
            .get(&self.url(path), Some(bearer_headers(token).unwrap()))
            .await
            .unwrap_or_else(|e| panic!("GET {path}: {e:?}"));
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    /// GET `path` without credentials, as a peer reads the well-known documents.
    pub async fn get_unauthenticated(&self, path: &str) -> Value {
        http_client()
            .get_json(&self.url(path), None)
            .await
            .unwrap_or_else(|e| panic!("GET {path}: {e:?}"))
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn common_config(port: u16) -> CommonConfig {
        serde_json::from_value(json!({
            "hosts": {
                "http": {"protocol": "http", "url": "127.0.0.1", "port": port.to_string(), "internal_port": port.to_string()},
                "grpc": null,
                "graphql": null
            },
            "db": {"db_type": "Postgres", "url": "localhost", "port": "5432"},
            "api": {"version": "v1", "openapi_path": "/openapi.json"},
            "connection": {"is_local": true, "is_prod": false, "is_vault_real": false, "has_proxy": false},
            "jwt_secret": "integration-tests"
        }))
        .expect("valid common config")
    }

    /// Cache section: the Redis at `REDIS_URL` (`redis://user:pass@host:port`), or Noop if unset.
    fn cache_config() -> Value {
        let Ok(url) = std::env::var("REDIS_URL") else {
            return json!({"cache_type": "Noop", "url": "", "port": "", "user": "", "password": ""});
        };
        let rest = url.trim_start_matches("redis://");
        let (auth, host) = rest.rsplit_once('@').unwrap_or(("", rest));
        let (user, password) = auth.split_once(':').unwrap_or(("", auth));
        let (host, port) = host.split_once(':').unwrap_or((host, "6379"));
        json!({"cache_type": "Redis", "url": host, "port": port, "user": user, "password": password})
    }

    /// The participant itself, as the other agents of its process know it.
    fn known(port: u16) -> Value {
        json!({
            "hosts": {
                "http": {"protocol": "http", "url": "127.0.0.1", "port": port.to_string(), "internal_port": port.to_string()},
                "grpc": null,
                "graphql": null
            },
            "api_version": "v1"
        })
    }

    /// An agent's config section: the common block plus its own fields.
    fn section<T: serde::de::DeserializeOwned>(common: &CommonConfig, own: Value) -> T {
        let mut section = own;
        section["common"] = serde_json::to_value(common).unwrap();
        serde_json::from_value(section).expect("valid agent config")
    }

    /// File-backed vault the negotiation flow never reads; built once per process from env.
    fn vault() -> VaultService {
        static ENV: Once = Once::new();
        ENV.call_once(|| {
            std::env::set_var("VAULT_PATH", std::env::temp_dir());
            std::env::set_var("VAULT_APP_DB", "db.json");
        });
        VaultService::Fake(FakeVaultService::new().expect("fake vault"))
    }
}

impl Drop for Participant {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}
