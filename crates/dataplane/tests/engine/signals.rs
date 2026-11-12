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

//! Control-plane signaling (`DataplaneSignal::{Completed, Failed}`), `FINITE` single-shot
//! quota enforcement (`410 Gone`), `maxRetries` transient-error budgeting, anti-DoS ingress
//! isolation, and `DataplaneManager` hook-failure signaling + idempotent termination.

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::any;
use axum::Router;
use common::test_utils::config_fixtures::transfer_config_fixture;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, HttpSpec, InteractionConfig,
    MockConnectorInstanceFacadeTrait, ProtocolSpec, PushLifecycle, TemplateVecString,
    TransferLifetime,
};
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::transfer_event::MockTransferEventRepo;
use dataplane::data::sea_orm::orm::{dataplane_transfers, transfer_event};
use dataplane::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxyEgress, DataplaneProxyIngress,
};
use dataplane::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, DeliveryPolicy, ResolvedAuthCredentials,
};
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::proxy::HttpDataProxy;
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::{
    DataplaneAddress, DataplaneCommand, DataplaneContinuation, DataplaneInitCommandDirection,
    DataplaneInitCommandTypes, DataplaneManager, DataplaneSignal, DataplaneSignalSender,
};
use http_body_util::BodyExt;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower::ServiceExt;
use urn::Urn;
use ymir::oauth::Owner;
use ymir::types::oauth::UserInfo;

const DP_ID: &str = "urn:dataplane-transfer:sig-1";
const TP_ID: &str = "urn:transfer-process:sig-tp-1";

fn mock_repo() -> Arc<MockDataplaneRepoTrait> {
    let mut repo = MockDataplaneRepoTrait::new();
    repo.expect_get_transfer_events_repo().returning(|| {
        let mut events_repo = MockTransferEventRepo::new();
        events_repo
            .expect_create_transfer_event()
            .returning(|ev| {
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

fn transfer_dto(
    role: TransferRole,
    mode: InteractionMode,
    upstream_url: &str,
    lifetime: TransferLifetime,
    max_retries: u32,
) -> DataplaneTransferDto {
    let runtime = DataplaneRuntime {
        auth: ResolvedAuthCredentials::NoAuth,
        subscription: serde_json::Value::Null,
        unsubscription: serde_json::Value::Null,
        delivery_policy: DeliveryPolicy {
            lifetime,
            max_retries,
        },
    };
    DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            id: DP_ID.to_string(),
            transfer_process_id: TP_ID.to_string(),
            role,
            interaction_mode: mode,
            state: TransferState::Started,
            connector_instance_id: Some("urn:connector:1".to_string()),
            ingress_config: serde_json::to_value(DataplaneProxyIngress::HttpListener {
                path: format!("/api/v1/data/{DP_ID}"),
                token_type: Some("Bearer".to_string()),
                token: Some("cap-token".to_string()),
            })
            .unwrap(),
            egress_config: serde_json::to_value(DataplaneProxyEgress::HttpProxy {
                path: upstream_url.to_string(),
                token_type: None,
                token: None,
            })
            .unwrap(),
            flow_control: Some(serde_json::to_value(runtime).unwrap()),
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }
}

async fn spawn_scripted_upstream(
    statuses: Vec<(StatusCode, &'static str)>,
) -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let scripted = Arc::new(statuses);
    let app = Router::new()
        .route(
            "/data",
            any(
                |State((hits, scripted)): State<(
                    Arc<AtomicUsize>,
                    Arc<Vec<(StatusCode, &'static str)>>,
                )>| async move {
                    let idx = hits.fetch_add(1, Ordering::SeqCst);
                    let &(status, body) = scripted
                        .get(idx)
                        .or_else(|| scripted.last())
                        .unwrap_or(&(StatusCode::OK, "ok"));
                    (status, body)
                },
            ),
        )
        .with_state((hits.clone(), scripted));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}/data"), hits)
}

#[tokio::test]
async fn provider_pull_finite_emits_completed_on_eof_and_blocks_second_call_with_410() {
    let (upstream_url, hits) = spawn_scripted_upstream(vec![(StatusCode::OK, "payload-bytes")]).await;
    let dto = transfer_dto(
        TransferRole::Provider,
        InteractionMode::Pull,
        &upstream_url,
        TransferLifetime::Finite,
        0,
    );
    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one()
        .returning(move |_, _| Ok(dto.clone()));

    let (tx, mut rx) = DataplaneSignalSender::open(8);
    let router = HttpDataProxy::new(Arc::new(svc), mock_repo())
        .with_allow_private_egress(true)
        .with_signals(tx)
        .router();

    // Unauthenticated callers must get 401/403, never emit Failed, and never burn the finite slot.
    let unauth_res = router
        .clone()
        .oneshot(Request::builder().uri(format!("/{DP_ID}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);
    assert!(rx.try_recv().is_err());

    // 1st valid GET: streams body to EOF and emits Completed once.
    let first_res = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}"))
                .header("authorization", "Bearer cap-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_res.status(), StatusCode::OK);
    let bytes = first_res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"payload-bytes");

    let sig = rx.try_recv().expect("Completed signal after EOF");
    assert_eq!(
        sig,
        DataplaneSignal::Completed {
            transfer_process_id: Urn::from_str(TP_ID).unwrap(),
        }
    );
    assert!(rx.try_recv().is_err());
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    // 2nd valid GET: rejected immediately with 410 Gone before touching upstream!
    let second_res = router
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}"))
                .header("authorization", "Bearer cap-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_res.status(), StatusCode::GONE);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn provider_push_finite_emits_completed_on_2xx_and_blocks_duplicate_push() {
    let (sink_url, hits) = spawn_scripted_upstream(vec![(StatusCode::ACCEPTED, "accepted")]).await;
    let dto = transfer_dto(
        TransferRole::Provider,
        InteractionMode::Push,
        &sink_url,
        TransferLifetime::Finite,
        0,
    );
    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one()
        .returning(move |_, _| Ok(dto.clone()));

    let (tx, mut rx) = DataplaneSignalSender::open(8);
    let router = HttpDataProxy::new(Arc::new(svc), mock_repo())
        .with_allow_private_egress(true)
        .with_signals(tx)
        .router();

    let res = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/{DP_ID}?ingress_token=cap-token"))
                .body(Body::from(r#"{"event":"once"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    assert_eq!(
        rx.try_recv().unwrap(),
        DataplaneSignal::Completed {
            transfer_process_id: Urn::from_str(TP_ID).unwrap(),
        }
    );

    let dup = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/{DP_ID}?ingress_token=cap-token"))
                .body(Body::from(r#"{"event":"twice"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dup.status(), StatusCode::GONE);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn transient_5xx_respects_max_retries_and_resets_after_200_ok() {
    let (upstream_url, _) = spawn_scripted_upstream(vec![
        (StatusCode::SERVICE_UNAVAILABLE, "blip-1"),
        (StatusCode::SERVICE_UNAVAILABLE, "blip-2"),
        (StatusCode::OK, "recovered"),
        (StatusCode::BAD_GATEWAY, "down-1"),
        (StatusCode::BAD_GATEWAY, "down-2"),
        (StatusCode::BAD_GATEWAY, "down-3"),
    ])
    .await;

    let dto = transfer_dto(
        TransferRole::Provider,
        InteractionMode::Pull,
        &upstream_url,
        TransferLifetime::Infinite,
        2, // tolerates 2 consecutive transient errors; fails on the 3rd consecutive one
    );
    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one()
        .returning(move |_, _| Ok(dto.clone()));

    let (tx, mut rx) = DataplaneSignalSender::open(8);
    let router = HttpDataProxy::new(Arc::new(svc), mock_repo())
        .with_allow_private_egress(true)
        .with_signals(tx)
        .router();

    let call = || {
        let router = router.clone();
        async move {
            router
                .oneshot(
                    Request::builder()
                        .uri(format!("/{DP_ID}"))
                        .header("authorization", "Bearer cap-token")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
                .status()
        }
    };

    // 2 transient blips within maxRetries=2 -> no Failed signal; 200 OK resets counter.
    assert_eq!(call().await, StatusCode::SERVICE_UNAVAILABLE);
    assert!(rx.try_recv().is_err());
    assert_eq!(call().await, StatusCode::SERVICE_UNAVAILABLE);
    assert!(rx.try_recv().is_err());
    assert_eq!(call().await, StatusCode::OK);
    assert!(rx.try_recv().is_err());

    // Now 3 consecutive 502s -> signal emitted on the 3rd failure!
    assert_eq!(call().await, StatusCode::BAD_GATEWAY);
    assert!(rx.try_recv().is_err());
    assert_eq!(call().await, StatusCode::BAD_GATEWAY);
    assert!(rx.try_recv().is_err());
    assert_eq!(call().await, StatusCode::BAD_GATEWAY);

    match rx.try_recv().expect("Failed signal after exceeding maxRetries") {
        DataplaneSignal::Failed {
            transfer_process_id,
            code,
            ..
        } => {
            assert_eq!(transfer_process_id, Urn::from_str(TP_ID).unwrap());
            assert_eq!(code, "DATAPLANE_MAX_RETRIES_EXCEEDED");
        }
        other => panic!("expected Failed signal, got {other:?}"),
    }
}

#[tokio::test]
async fn infinite_pull_functional_404_never_fails_but_backend_401_fails_immediately() {
    let (upstream_url, _) = spawn_scripted_upstream(vec![
        (StatusCode::NOT_FOUND, "missing item"),
        (StatusCode::UNAUTHORIZED, "revoked api key"),
    ])
    .await;

    let dto = transfer_dto(
        TransferRole::Provider,
        InteractionMode::Pull,
        &upstream_url,
        TransferLifetime::Infinite,
        5, // even with maxRetries=5, 401 from provider's own backend is permanent
    );
    let mut svc = MockDataplaneTransferServiceTrait::new();
    svc.expect_get_one()
        .returning(move |_, _| Ok(dto.clone()));

    let (tx, mut rx) = DataplaneSignalSender::open(8);
    let router = HttpDataProxy::new(Arc::new(svc), mock_repo())
        .with_allow_private_egress(true)
        .with_signals(tx)
        .router();

    // 404 on INFINITE pull: functional client query error -> NO signal.
    let res_404 = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}"))
                .header("authorization", "Bearer cap-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_404.status(), StatusCode::NOT_FOUND);
    assert!(rx.try_recv().is_err());

    // 401 on (Provider, Pull): backend rejected connector credentials -> immediate Failed!
    let res_401 = router
        .oneshot(
            Request::builder()
                .uri(format!("/{DP_ID}"))
                .header("authorization", "Bearer cap-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_401.status(), StatusCode::UNAUTHORIZED);
    match rx.try_recv().expect("immediate Failed on backend 401") {
        DataplaneSignal::Failed { code, .. } => {
            assert_eq!(code, "DATAPLANE_UPSTREAM_AUTH_REJECTED");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn subscribe_hook_failure_emits_failed_and_termination_is_idempotent() {
    // Bind and immediately drop a port so webhook subscription deterministically fails.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_port = listener.local_addr().unwrap().port();
    drop(listener);

    let connector_urn = Urn::from_str("urn:connector:push-broken").unwrap();
    let tp_urn = Urn::from_str(TP_ID).unwrap();
    let connector = ConnectorInstanceDto {
        id: connector_urn.clone(),
        user_id: "tenant-1".to_string(),
        metadata: ConnectorMetadata {
            name: Some("broken-push".to_string()),
            author: None,
            description: None,
            version: Some("1.0.0".to_string()),
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime: TransferLifetime::Finite,
            max_retries: Some(0),
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: format!("http://127.0.0.1:{dead_port}/hooks"),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: Some(r#"{"cb":"{{__RUNTIME_INGRESS__}}"}"#.to_string()),
            }),
            unsubscribe: None,
        }),
        distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
    };

    let persisted: Arc<Mutex<Option<DataplaneTransferDto>>> = Arc::new(Mutex::new(None));
    let edits_after_terminated = Arc::new(AtomicUsize::new(0));

    let mut svc = MockDataplaneTransferServiceTrait::new();
    let persisted_create = persisted.clone();
    svc.expect_create().returning(move |_, new_dto| {
        let dto = DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                user_id: "tenant-1".to_string(),
                user_role: ymir::types::oauth::RolePath::root(),
                visibility: ymir::types::participants::Visibility::Private,
                id: new_dto.id.as_ref().unwrap().to_string(),
                transfer_process_id: new_dto.transfer_process_id.clone(),
                role: new_dto.role.clone(),
                interaction_mode: new_dto.interaction_mode.clone(),
                state: new_dto.state.clone(),
                connector_instance_id: new_dto
                    .connector_instance_id
                    .as_ref()
                    .map(ToString::to_string),
                ingress_config: new_dto.ingress_config.clone(),
                egress_config: new_dto.egress_config.clone(),
                flow_control: None,
                created_at: chrono::Utc::now().into(),
                updated_at: None,
            },
            fields: Default::default(),
            logs: vec![],
        };
        *persisted_create.lock().unwrap() = Some(dto.clone());
        Ok(dto)
    });
    let persisted_get = persisted.clone();
    svc.expect_get_by_process_id()
        .returning(move |_, _| Ok(persisted_get.lock().unwrap().clone().unwrap()));
    let persisted_edit = persisted.clone();
    let edits_counter = edits_after_terminated.clone();
    svc.expect_edit().returning(move |_, _, edit_dto| {
        let mut guard = persisted_edit.lock().unwrap();
        let dto = guard.as_mut().unwrap();
        if dto.inner.state == TransferState::Terminated {
            edits_counter.fetch_add(1, Ordering::SeqCst);
        }
        if let Some(st) = &edit_dto.state {
            dto.inner.state = st.clone();
        }
        if let Some(ing) = &edit_dto.ingress_config {
            dto.inner.ingress_config = ing.clone();
        }
        if let Some(egr) = &edit_dto.egress_config {
            dto.inner.egress_config = egr.clone();
        }
        if let Some(fc) = &edit_dto.flow_control {
            dto.inner.flow_control = Some(fc.clone());
        }
        Ok(dto.clone())
    });

    let mut conn_facade = MockConnectorInstanceFacadeTrait::new();
    let connector_clone = connector.clone();
    conn_facade
        .expect_get_instance_by_id()
        .returning(move |_| Ok(Some(connector_clone.clone())));

    let (tx, mut rx) = DataplaneSignalSender::open(8);
    let mgr = DataplaneManager::new(
        Arc::new(svc),
        Arc::new(conn_facade),
        transfer_config_fixture(),
    )
    .with_signals(tx);

    // SetInit succeeds and persists DeliveryPolicy { lifetime: Finite, max_retries: 0 }.
    mgr.execute_command(DataplaneCommand::SetInit(
        DataplaneInitCommandTypes::AsProvider {
            owner: Owner::private(&UserInfo::system()),
            transfer_process_id: tp_urn.clone(),
            connector_instance: connector,
            direction: DataplaneInitCommandDirection::Push {
                data_address: Some(DataplaneAddress {
                    endpoint_type: "HTTP".to_string(),
                    endpoint: "http://consumer.example.com/sink".to_string(),
                    authorization_type: None,
                    authorization: None,
                }),
            },
        },
    ))
    .await
    .unwrap();
    assert!(rx.try_recv().is_err());

    // SetSubscribing fails against dead webhook -> emits Failed and terminates dataplane!
    let err = mgr
        .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
            transfer_dto_urn: tp_urn.clone(),
        }))
        .await;
    assert!(err.is_err());

    match rx.try_recv().expect("Failed signal from SetSubscribing") {
        DataplaneSignal::Failed {
            transfer_process_id,
            code,
            ..
        } => {
            assert_eq!(transfer_process_id, tp_urn);
            assert_eq!(code, "DATAPLANE_SUBSCRIBE_FAILED");
        }
        other => panic!("expected Failed signal, got {other:?}"),
    }
    assert_eq!(
        persisted.lock().unwrap().as_ref().unwrap().inner.state,
        TransferState::Terminated
    );

    // Subsequent SetTerminating (e.g. from control-plane on_terminate_post) is an idempotent no-op!
    mgr.execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
        transfer_dto_urn: tp_urn,
    }))
    .await
    .unwrap();
    assert_eq!(edits_after_terminated.load(Ordering::SeqCst), 0);
}
