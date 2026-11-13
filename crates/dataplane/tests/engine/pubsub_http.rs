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

//! Regression tests for BUG-1 (`flow_control.subscription` persistence across restarts)
//! and BUG-4 (`HttpPubSubscriber` vault resolution, ApiKey/headers, and `?ingress_token=`).

use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, post};
use axum::{Json, Router};
use common::test_utils::config_fixtures::transfer_config_fixture;
use connector::entities::parameters::TemplateMapString;
use connector::{
    ApiKeyLocation, AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, HttpSpec,
    InteractionConfig, KeystoreLookup, MockConnectorInstanceFacadeTrait, ProtocolSpec,
    PushLifecycle, SecretSource, SecretString, TemplateVecString,
};
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes,
};
use dataplane::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactory;
use dataplane::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use dataplane::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::DataplaneAddress;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use urn::Urn;

#[derive(Clone, Default)]
struct WebhookBackendState {
    subscribe_uri: Arc<Mutex<String>>,
    subscribe_headers: Arc<Mutex<HashMap<String, String>>>,
    subscribe_body: Arc<Mutex<Value>>,
    unsubscribed_id: Arc<Mutex<Option<String>>>,
}

async fn spawn_webhook_backend() -> (String, WebhookBackendState) {
    let state = WebhookBackendState::default();
    let app = Router::new()
        .route(
            "/hooks",
            post(
                |State(st): State<WebhookBackendState>,
                 OriginalUri(uri): OriginalUri,
                 headers: HeaderMap,
                 Json(body): Json<Value>| async move {
                    *st.subscribe_uri.lock().unwrap() = uri.to_string();
                    let map = headers
                        .iter()
                        .filter_map(|(k, v)| {
                            v.to_str().ok().map(|val| (k.as_str().to_string(), val.to_string()))
                        })
                        .collect();
                    *st.subscribe_headers.lock().unwrap() = map;
                    *st.subscribe_body.lock().unwrap() = body;
                    (
                        StatusCode::CREATED,
                        Json(json!({ "id": "sub-99" })),
                    )
                },
            ),
        )
        .route(
            "/hooks/{id}",
            delete(
                |State(st): State<WebhookBackendState>, Path(id): Path<String>| async move {
                    *st.unsubscribed_id.lock().unwrap() = Some(id);
                    StatusCode::NO_CONTENT
                },
            ),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), state)
}

struct StubKeystoreLookup;

#[async_trait::async_trait]
impl KeystoreLookup for StubKeystoreLookup {
    async fn get_parameter(&self, _user_id: &str, _key: &str) -> Option<Value> {
        None
    }

    async fn get_secret(&self, _user_id: &str, key: &str) -> Option<Value> {
        if key == "/runtime/urn:dataplane-transfer:push-1/oauth_access_token" {
            Some(json!("resolved-vault-oauth-token"))
        } else {
            None
        }
    }
}

#[tokio::test]
async fn set_subscribing_persists_subscription_and_unsubscribe_resolves_runtime_json() {
    let (base_url, backend) = spawn_webhook_backend().await;
    let tp_urn = Urn::from_str("urn:transfer-process:push-1").unwrap();
    let conn_urn = Urn::from_str("urn:connector-instance:push-1").unwrap();

    let mut headers_map = HashMap::new();
    headers_map.insert("x-custom-source".to_string(), "ecostars".to_string());

    let connector = ConnectorInstanceDto {
        id: conn_urn.clone(),
        user_id: "tenant-1".to_string(),
        metadata: ConnectorMetadata {
            name: Some("webhook-push".to_string()),
            author: None,
            description: None,
            version: Some("1.0".to_string()),
            created_at: None,
        },
        authentication_config: AuthenticationConfig::ApiKey {
            key: "api_key".to_string(),
            value: SecretString {
                source: SecretSource::Plain("hook-secret-123".to_string()),
            },
            location: ApiKeyLocation::Query,
        },
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: Default::default(),
            max_retries: None,
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: format!("{base_url}/hooks"),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: Some(TemplateMapString::Value(headers_map)),
                body_template: Some(
                    r#"{"callback":"http://provider.local{{__RUNTIME_INGRESS__}}"}"#.to_string(),
                ),
            }),
            unsubscribe: Some(ProtocolSpec::Http(HttpSpec {
                url_template: format!(
                    "{base_url}/hooks/{{{{__RUNTIME_JSON_{{.subscription.id}}__}}}}"
                ),
                method: TemplateVecString::Value(vec!["DELETE".to_string()]),
                headers: None,
                body_template: None,
            })),
        }),
        distribution_id: Urn::from_str("urn:distribution:push-1").unwrap(),
    };

    // Shared in-memory transfer row simulating DB persistence across commands & restarts.
    let persisted = Arc::new(Mutex::new(DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            id: "urn:dataplane-transfer:push-1".to_string(),
            transfer_process_id: tp_urn.to_string(),
            role: TransferRole::Provider,
            interaction_mode: InteractionMode::Push,
            state: TransferState::Init,
            connector_instance_id: Some(conn_urn.to_string()),
            ingress_config: json!("NoOp"),
            egress_config: json!("NoOp"),
            flow_control: None,
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }));

    let mut svc = MockDataplaneTransferServiceTrait::new();
    let p_create = persisted.clone();
    svc.expect_create()
        .returning(move |_, _| Ok(p_create.lock().unwrap().clone()));
    let p_get = persisted.clone();
    svc.expect_get_by_process_id()
        .returning(move |_, _| Ok(p_get.lock().unwrap().clone()));
    let p_edit = persisted.clone();
    svc.expect_edit().returning(move |_, _, edit| {
        let mut guard = p_edit.lock().unwrap();
        if let Some(st) = &edit.state {
            guard.inner.state = st.clone();
        }
        if let Some(ing) = &edit.ingress_config {
            guard.inner.ingress_config = ing.clone();
        }
        if let Some(eg) = &edit.egress_config {
            guard.inner.egress_config = eg.clone();
        }
        if let Some(fc) = &edit.flow_control {
            guard.inner.flow_control = Some(fc.clone());
        }
        Ok(guard.clone())
    });

    let mut conn_facade = MockConnectorInstanceFacadeTrait::new();
    let conn_clone = connector.clone();
    conn_facade
        .expect_get_instance_by_id()
        .returning(move |_| Ok(Some(conn_clone.clone())));

    let factory = Arc::new(
        DataplaneDriverFactory::new().with_keystore(Arc::new(StubKeystoreLookup)),
    );
    let manager = DataplaneManager::new(
        Arc::new(svc),
        Arc::new(conn_facade),
        transfer_config_fixture(),
    )
    .with_driver_factory(factory);

    // 1. SetInit -> Ready (provisions HttpListener with per-transfer ingress token + ApiKey auth)
    manager
        .execute_command(DataplaneCommand::SetInit(
            DataplaneInitCommandTypes::AsProvider {
                owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
                transfer_process_id: tp_urn.clone(),
                connector_instance: connector,
                direction: DataplaneInitCommandDirection::Push {
                    data_address: Some(DataplaneAddress {
                        endpoint_type: "HTTP".to_string(),
                        endpoint: "http://consumer.local/sink".to_string(),
                        authorization_type: Some("Bearer".to_string()),
                        authorization: Some("sink-tok".to_string()),
                    }),
                },
            },
        ))
        .await
        .unwrap();

    // 2. SetSubscribing (rehydrated from persisted row) -> calls POST /hooks, saves subscription in flow_control!
    manager
        .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation::new(tp_urn.clone())))
        .await
        .unwrap();

    {
        let stored = persisted.lock().unwrap();
        assert_eq!(stored.inner.state, TransferState::Started);
        let rt: DataplaneRuntime =
            serde_json::from_value(stored.inner.flow_control.clone().expect("flow_control saved"))
                .expect("valid DataplaneRuntime");
        assert_eq!(
            rt.subscription,
            json!({ "id": "sub-99" }),
            "BUG-1 fix: subscription response must be persisted in flow_control on SetStarted"
        );
    }

    // Check BUG-4 + AP-1.6 artifacts received by backend on subscribe:
    assert_eq!(
        *backend.subscribe_uri.lock().unwrap(),
        "/hooks?api_key=hook-secret-123"
    );
    assert_eq!(
        backend
            .subscribe_headers
            .lock()
            .unwrap()
            .get("x-custom-source")
            .map(String::as_str),
        Some("ecostars")
    );
    let callback_str = backend.subscribe_body.lock().unwrap()["callback"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        callback_str.contains("/dataplane/proxy/urn:dataplane-transfer:push-1?ingress_token="),
        "callback URL should embed per-transfer ingress_token: {callback_str}"
    );

    // Simulate vaulted OAuth2 placeholder in flow_control before unsubscribe to verify BUG-4 vault lookup:
    {
        let mut guard = persisted.lock().unwrap();
        let rt = DataplaneRuntime {
            auth: ResolvedAuthCredentials::OAuth2 {
                access_token:
                    "{{__RUNTIME_SECRET_{/runtime/urn:dataplane-transfer:push-1/oauth_access_token}__}}"
                        .to_string(),
                token_type: "Bearer".to_string(),
                expires_at: None,
                refresh_token: None,
            },
            subscription: json!({ "id": "sub-99" }),
            unsubscription: Value::Null,
            delivery_policy: Default::default(),
        };
        guard.inner.flow_control = Some(serde_json::to_value(rt).unwrap());
    }

    // 3. SetUnsubscribing (rehydrated from persisted row) -> resolves {{__RUNTIME_JSON_{.subscription.id}__}} -> sub-99!
    manager
        .execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation::new(tp_urn)))
        .await
        .unwrap();

    assert_eq!(
        backend.unsubscribed_id.lock().unwrap().as_deref(),
        Some("sub-99"),
        "unsubscribe must resolve {{__RUNTIME_JSON_{{.subscription.id}}__}} from persisted flow_control"
    );
    assert_eq!(persisted.lock().unwrap().inner.state, TransferState::Stopped);
}
