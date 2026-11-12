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

//! Security, credential-isolation, and redaction tests for `HttpDataProxy`.

use axum::body::Body;
use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::routing::any;
use axum::Router;
use connector::ApiKeyLocation;
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::transfer_event::MockTransferEventRepo;
use dataplane::data::sea_orm::orm::{dataplane_transfers, transfer_event};
use dataplane::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxyEgress, DataplaneProxyIngress,
};
use dataplane::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::proxy::HttpDataProxy;
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower::ServiceExt;

const DP_ID: &str = "urn:dataplane-transfer:proxy-sec-1";

#[derive(Clone, Default)]
struct CapturedUpstream {
    uri: Arc<Mutex<String>>,
    auth_header: Arc<Mutex<Option<String>>>,
}

async fn spawn_upstream() -> (String, CapturedUpstream) {
    let captured = CapturedUpstream::default();
    let app = Router::new()
        .route(
            "/{*rest}",
            any(
                |State(cap): State<CapturedUpstream>,
                 OriginalUri(uri): OriginalUri,
                 headers: HeaderMap| async move {
                    *cap.uri.lock().unwrap() = uri.to_string();
                    *cap.auth_header.lock().unwrap() = headers
                        .get("authorization")
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_owned);
                    (StatusCode::OK, "ok")
                },
            ),
        )
        .with_state(captured.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}/upstream"), captured)
}

fn build_transfer_dto(
    role: TransferRole,
    mode: InteractionMode,
    ingress: DataplaneProxyIngress,
    egress: DataplaneProxyEgress,
    runtime: Option<DataplaneRuntime>,
) -> DataplaneTransferDto {
    DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            id: DP_ID.to_string(),
            transfer_process_id: "urn:transfer-process:proxy-sec-1".to_string(),
            role,
            interaction_mode: mode,
            state: TransferState::Started,
            connector_instance_id: Some("urn:connector:1".to_string()),
            ingress_config: serde_json::to_value(ingress).unwrap(),
            egress_config: serde_json::to_value(egress).unwrap(),
            flow_control: runtime.map(|rt| serde_json::to_value(rt).unwrap()),
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }
}

fn mock_repo_recording_events(
    recorded: Arc<Mutex<Vec<transfer_event::NewTransferEvent>>>,
) -> Arc<MockDataplaneRepoTrait> {
    let mut repo = MockDataplaneRepoTrait::new();
    repo.expect_get_transfer_events_repo().returning(move || {
        let recorded = recorded.clone();
        let mut events_repo = MockTransferEventRepo::new();
        events_repo
            .expect_create_transfer_event()
            .returning(move |ev| {
                recorded.lock().unwrap().push(ev.clone());
                Ok(transfer_event::Model {
                    id: "urn:transfer-event:1".to_string(),
                    user_id: ev.owner.user_id.clone(),
                    user_role: ev.owner.role.clone(),
                    visibility: ev.owner.visibility.clone(),
                    transfer_id: ev.transfer_id.clone(),
                    level: ev.level.clone(),
                    component: ev.component.clone(),
                    message: ev.message.clone(),
                    data: ev.data.clone(),
                    created_at: chrono::Utc::now().into(),
                })
            });
        Arc::new(events_repo)
    });
    Arc::new(repo)
}

#[tokio::test]
async fn rejects_missing_and_invalid_ingress_token_with_401_and_403() {
    let dto = build_transfer_dto(
        TransferRole::Provider,
        InteractionMode::Pull,
        DataplaneProxyIngress::HttpListener {
            path: "/dataplane/proxy/urn:dataplane-transfer:proxy-sec-1".to_string(),
            token_type: Some("Bearer".to_string()),
            token: Some("expected-secret-token".to_string()),
        },
        DataplaneProxyEgress::HttpProxy {
            path: "http://127.0.0.1:9/api".to_string(),
            token_type: None,
            token: None,
        },
        None,
    );

    let mut svc = MockDataplaneTransferServiceTrait::new();
    let dto_clone = dto.clone();
    svc.expect_get_one()
        .returning(move |_, _| Ok(dto_clone.clone()));
    let repo = mock_repo_recording_events(Arc::new(Mutex::new(Vec::new())));

    let router = HttpDataProxy::new(Arc::new(svc), repo)
        .with_allow_private_egress(true)
        .router();

    // 1. Missing token -> 401 Unauthorized
    let res = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}/items"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Wrong Bearer token -> 403 Forbidden
    let res = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}/items"))
                .header("Authorization", "Bearer wrong-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 3. Wrong query ingress_token -> 403 Forbidden
    let res = router
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}/items?ingress_token=wrong"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn accepts_bearer_and_query_ingress_token_and_strips_query_token_upstream() {
    let (upstream_url, captured) = spawn_upstream().await;
    let dto = build_transfer_dto(
        TransferRole::Provider,
        InteractionMode::Pull,
        DataplaneProxyIngress::HttpListener {
            path: format!("/dataplane/proxy/{DP_ID}"),
            token_type: Some("Bearer".to_string()),
            token: Some("cap-tok-123".to_string()),
        },
        DataplaneProxyEgress::HttpProxy {
            path: upstream_url,
            token_type: None,
            token: None,
        },
        Some(DataplaneRuntime {
            auth: ResolvedAuthCredentials::BearerToken {
                token: "backend-secret".to_string(),
            },
            subscription: Value::Null,
            unsubscription: Value::Null,
        }),
    );

    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one().returning(move |_, _| Ok(dto.clone()));
    let repo = mock_repo_recording_events(Arc::new(Mutex::new(Vec::new())));

    let router = HttpDataProxy::new(Arc::new(svc), repo)
        .with_allow_private_egress(true)
        .router();

    // Valid query ingress_token + user query param -> ingress_token stripped, backend auth injected
    let res = router
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}/records?ingress_token=cap-tok-123&page=2"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(*captured.uri.lock().unwrap(), "/upstream/records?page=2");
    assert_eq!(
        captured.auth_header.lock().unwrap().as_deref(),
        Some("Bearer backend-secret")
    );
}

#[tokio::test]
async fn blocks_private_loopback_and_metadata_egress_by_default() {
    for blocked_target in [
        "http://127.0.0.1:8080/internal",
        "http://169.254.169.254/latest/meta-data/",
        "http://10.0.0.5/admin",
        "http://localhost:9000/secret",
        "ftp://93.184.216.34/file",
    ] {
        let dto = build_transfer_dto(
            TransferRole::Provider,
            InteractionMode::Pull,
            DataplaneProxyIngress::NoOp,
            DataplaneProxyEgress::HttpProxy {
                path: blocked_target.to_string(),
                token_type: None,
                token: None,
            },
            None,
        );
        let mut svc = MockDataplaneTransferServiceTrait::new();
        svc.expect_get_one().returning(move |_, _| Ok(dto.clone()));
        let repo = mock_repo_recording_events(Arc::new(Mutex::new(Vec::new())));

        // Default HttpDataProxy::new has allow_private_egress = false!
        let router = HttpDataProxy::new(Arc::new(svc), repo).router();
        let res = router
            .oneshot(
                Request::builder()
                    .uri(format!("/{DP_ID}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "target {blocked_target} should be blocked by anti-SSRF"
        );
    }
}

#[tokio::test]
async fn provider_push_uses_consumer_egress_token_and_never_leaks_provider_backend_auth() {
    let (consumer_sink_url, captured) = spawn_upstream().await;
    let dto = build_transfer_dto(
        TransferRole::Provider,
        InteractionMode::Push,
        DataplaneProxyIngress::HttpListener {
            path: format!("/dataplane/proxy/{DP_ID}"),
            token_type: Some("Bearer".to_string()),
            token: Some("provider-webhook-ingress-cap".to_string()),
        },
        DataplaneProxyEgress::HttpProxy {
            path: format!("{consumer_sink_url}/sink"),
            token_type: Some("Bearer".to_string()),
            token: Some("consumer-sink-ingress-token".to_string()),
        },
        Some(DataplaneRuntime {
            auth: ResolvedAuthCredentials::BearerToken {
                token: "PROVIDER-TOP-SECRET-BACKEND-TOKEN".to_string(),
            },
            subscription: json!({"id": "sub-1"}),
            unsubscription: Value::Null,
        }),
    );

    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one().returning(move |_, _| Ok(dto.clone()));
    let repo = mock_repo_recording_events(Arc::new(Mutex::new(Vec::new())));

    let router = HttpDataProxy::new(Arc::new(svc), repo)
        .with_allow_private_egress(true)
        .router();

    let res = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/{DP_ID}?ingress_token=provider-webhook-ingress-cap"
                ))
                .body(Body::from(r#"{"event":"temperature","val":21.5}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        captured.auth_header.lock().unwrap().as_deref(),
        Some("Bearer consumer-sink-ingress-token"),
        "Provider+Push must present consumer DataAddress token, never provider runtime.auth"
    );
}

#[tokio::test]
async fn redacts_sensitive_query_params_in_transfer_events() {
    let (upstream_url, _) = spawn_upstream().await;
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let dto = build_transfer_dto(
        TransferRole::Provider,
        InteractionMode::Pull,
        DataplaneProxyIngress::NoOp,
        DataplaneProxyEgress::HttpProxy {
            path: format!("{upstream_url}/query?token=raw-arcgis-secret&format=geojson"),
            token_type: None,
            token: None,
        },
        Some(DataplaneRuntime {
            auth: ResolvedAuthCredentials::ApiKey {
                key: "api_key".to_string(),
                value: "super-secret-key".to_string(),
                location: ApiKeyLocation::Query,
            },
            subscription: Value::Null,
            unsubscription: Value::Null,
        }),
    );

    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one().returning(move |_, _| Ok(dto.clone()));
    let repo = mock_repo_recording_events(recorded.clone());

    let router = HttpDataProxy::new(Arc::new(svc), repo)
        .with_allow_private_egress(true)
        .router();

    let res = router
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let events = recorded.lock().unwrap();
    assert_eq!(events.len(), 1);
    let serialized = serde_json::to_string(&events[0].data).unwrap();
    assert!(
        !serialized.contains("raw-arcgis-secret") && !events[0].message.contains("raw-arcgis-secret"),
        "raw query token must be redacted in transfer events: {serialized}"
    );
    assert!(
        !serialized.contains("super-secret-key") && !events[0].message.contains("super-secret-key"),
        "query API key must never appear in transfer events: {serialized}"
    );
}
