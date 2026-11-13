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

//! Continuations on consumer transfers, and commands the manager rejects.

use super::*;

/// SetStarted on a Stopped consumer pull transfer persists Started and answers Ok.
#[tokio::test]
async fn set_started() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:10").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:10",
                "urn:transfer-process:10",
                TransferRole::Consumer,
                InteractionMode::Pull,
                TransferState::Stopped,
                None,
            ))
        });

    mock_entity
        .expect_edit()
        .withf(|_, _, dto| dto.state == Some(TransferState::Started))
        .times(1)
        .returning(|_, id, dto| {
            Ok(dummy_dataplane_transfer_dto(
                id.as_ref(),
                "urn:transfer-process:10",
                TransferRole::Consumer,
                InteractionMode::Pull,
                dto.state.clone().unwrap(),
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(matches!(result.unwrap(), DataplaneCommandResponse::Ok));
}

/// ConsumerPull only accepts SetStarted when Stopped; calling SetStarted in Init is rejected.
#[tokio::test]
async fn set_started_rejected_when_consumer_pull_not_stopped() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:10b").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:10b",
                "urn:transfer-process:10b",
                TransferRole::Consumer,
                InteractionMode::Pull,
                TransferState::Init,
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(result.is_err());
}

/// SetStopped loads the transfer by process id and persists Stopped.
#[tokio::test]
async fn set_stopped() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:11").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:11",
                "urn:transfer-process:11",
                TransferRole::Consumer,
                InteractionMode::Pull,
                TransferState::Started,
                None,
            ))
        });

    mock_entity
        .expect_edit()
        .withf(|_, _, dto| dto.state == Some(TransferState::Stopped))
        .times(1)
        .returning(|_, id, dto| {
            Ok(dummy_dataplane_transfer_dto(
                id.as_ref(),
                "urn:transfer-process:11",
                TransferRole::Consumer,
                InteractionMode::Pull,
                dto.state.clone().unwrap(),
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetStopped(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(result.is_ok());
}

/// SetTerminating loads the transfer by process id and persists Terminated.
#[tokio::test]
async fn set_terminating() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:12").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:12",
                "urn:transfer-process:12",
                TransferRole::Consumer,
                InteractionMode::Pull,
                TransferState::Started,
                None,
            ))
        });

    mock_entity
        .expect_edit()
        .withf(|_, _, dto| dto.state == Some(TransferState::Terminated))
        .times(1)
        .returning(|_, id, dto| {
            Ok(dummy_dataplane_transfer_dto(
                id.as_ref(),
                "urn:transfer-process:12",
                TransferRole::Consumer,
                InteractionMode::Pull,
                dto.state.clone().unwrap(),
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(result.is_ok());
}

/// ConsumerPull rejects SetSubscribing.
#[tokio::test]
async fn set_subscribing_rejected_for_consumer_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:13").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:13",
                "urn:transfer-process:13",
                TransferRole::Consumer,
                InteractionMode::Pull,
                TransferState::Started,
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(result.is_err());
}

/// ConsumerPull rejects SetUnsubscribing.
#[tokio::test]
async fn set_unsubscribing_rejected_for_consumer_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:14").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:14",
                "urn:transfer-process:14",
                TransferRole::Consumer,
                InteractionMode::Pull,
                TransferState::Started,
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(result.is_err());
}

/// ConsumerPush resolves NoOpPubSubscriber on continuation and transitions Ready -> Subscribing -> Started.
#[tokio::test]
async fn set_subscribing_consumer_push() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:15").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:15",
                "urn:transfer-process:15",
                TransferRole::Consumer,
                InteractionMode::Push,
                TransferState::Ready,
                None,
            ))
        });

    for state in [TransferState::Subscribing, TransferState::Started] {
        let s = state.clone();
        mock_entity
            .expect_edit()
            .times(1)
            .withf(move |_, _, edit| edit.state == Some(s.clone()))
            .returning(move |_, _, _| {
                Ok(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:15",
                    "urn:transfer-process:15",
                    TransferRole::Consumer,
                    InteractionMode::Push,
                    state.clone(),
                    None,
                ))
            });
    }

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(matches!(result.unwrap(), DataplaneCommandResponse::Ok));
}

/// ConsumerPush transitions Started -> Unsubscribing -> Stopped on SetUnsubscribing, and
/// Started -> Unsubscribing -> Stopped -> Terminated on SetTerminating.
#[tokio::test]
async fn set_terminating_consumer_push_unsubscribes_before_terminated() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:16").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:16",
                "urn:transfer-process:16",
                TransferRole::Consumer,
                InteractionMode::Push,
                TransferState::Started,
                None,
            ))
        });

    for state in [
        TransferState::Unsubscribing,
        TransferState::Stopped,
        TransferState::Terminated,
    ] {
        let s = state.clone();
        mock_entity
            .expect_edit()
            .times(1)
            .withf(move |_, _, edit| edit.state == Some(s.clone()))
            .returning(move |_, _, _| {
                Ok(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:16",
                    "urn:transfer-process:16",
                    TransferRole::Consumer,
                    InteractionMode::Push,
                    state.clone(),
                    None,
                ))
            });
    }

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation::new(tp_id)))
    .await;

    assert!(matches!(result.unwrap(), DataplaneCommandResponse::Ok));
}

/// ConsumerPush rejects external SetStarted and SetStopped commands.
#[tokio::test]
async fn set_started_and_stopped_rejected_for_consumer_push() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:17").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(2)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:17",
                "urn:transfer-process:17",
                TransferRole::Consumer,
                InteractionMode::Push,
                TransferState::Ready,
                None,
            ))
        });

    let mgr = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    );

    assert!(mgr
        .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation::new(tp_id.clone())))
        .await
        .is_err());

    assert!(mgr
        .execute_command(DataplaneCommand::SetStopped(DataplaneContinuation::new(tp_id)))
        .await
        .is_err());
}

/// SetAuth is an internal state-machine step — never a valid external command.
#[tokio::test]
async fn unexpected_command_returns_err_response() {
    let mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetAuth)
    .await;

    assert!(result.is_err());
}

/// A continuation for an unknown transfer is an error.
#[tokio::test]
async fn continuation_not_found_returns_err() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Err(ymir::errors::Errors::missing_resource(
                "dataplane transfer",
                "not found",
                None,
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation::new(Urn::from_str("urn:transfer-process:99").unwrap())))
    .await;

    assert!(result.is_err());
}

/// ConsumerPull records redacted proxy configuration reasons across Configuring..Started and final termination.
#[tokio::test]
async fn consumer_pull_records_redacted_stage_reasons_and_termination_reason() {
    use std::sync::Mutex;

    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();
    let tp_id = Urn::from_str("urn:transfer-process:18").unwrap();

    let current = Arc::new(Mutex::new(dummy_dataplane_transfer_dto(
        "urn:dataplane-transfer:18",
        "urn:transfer-process:18",
        TransferRole::Consumer,
        InteractionMode::Pull,
        TransferState::Init,
        None,
    )));
    let recorded = Arc::new(Mutex::new(Vec::<(Option<TransferState>, Option<String>)>::new()));

    let current_get = current.clone();
    mock_entity
        .expect_get_by_process_id()
        .returning(move |_, _| Ok(current_get.lock().unwrap().clone()));

    let current_edit = current.clone();
    let recorded_edit = recorded.clone();
    mock_entity.expect_edit().returning(move |_, _, edit| {
        recorded_edit
            .lock()
            .unwrap()
            .push((edit.state.clone(), edit.reason.clone()));
        let mut guard = current_edit.lock().unwrap();
        if let Some(st) = &edit.state {
            guard.inner.state = st.clone();
        }
        if let Some(ing) = &edit.ingress_config {
            guard.inner.ingress_config = ing.clone();
        }
        if let Some(egr) = &edit.egress_config {
            guard.inner.egress_config = egr.clone();
        }
        Ok(guard.clone())
    });

    let mgr = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    );

    mgr.execute_command(DataplaneCommand::SetConfiguring((
        DataplaneContinuation::with_reason(
            tp_id.clone(),
            Some("Transfer started (DSP peer: TransferStartMessage)".to_string()),
        ),
        DataplaneAddress {
            endpoint_type: "HTTP".to_string(),
            endpoint: "https://user:secretpass@provider.example/dataplane/proxy/p1?ingress_token=leaked-q"
                .to_string(),
            authorization_type: Some("Bearer".to_string()),
            authorization: Some("secret-bearer-token".to_string()),
        },
    )))
    .await
    .unwrap();

    let term_reason = "Transfer terminated (DSP peer: TransferTerminationMessage) [code=DATAPLANE_FORBIDDEN_TARGET]: private IP blocked";
    mgr.execute_command(DataplaneCommand::SetStopped(
        DataplaneContinuation::with_reason(
            tp_id.clone(),
            Some(format!("Stopping before final transition; {term_reason}")),
        ),
    ))
    .await
    .unwrap();
    mgr.execute_command(DataplaneCommand::SetTerminating(
        DataplaneContinuation::with_reason(tp_id, Some(term_reason.to_string())),
    ))
    .await
    .unwrap();

    let entries = recorded.lock().unwrap().clone();
    assert_eq!(entries.len(), 6);

    assert_eq!(entries[0].0, Some(TransferState::Configuring));
    let cfg_reason = entries[0].1.as_deref().unwrap();
    assert!(cfg_reason.contains("HttpListener(/dataplane/proxy/urn:dataplane-transfer:18, auth=true)"));
    assert!(cfg_reason.contains("HttpProxy(https://provider.example/dataplane/proxy/p1, auth=true)"));
    assert!(cfg_reason.contains("Transfer started (DSP peer: TransferStartMessage)"));
    assert!(!cfg_reason.contains("secretpass"));
    assert!(!cfg_reason.contains("leaked-q"));
    assert!(!cfg_reason.contains("secret-bearer-token"));

    let persisted_ingress = current.lock().unwrap().inner.ingress_config.clone();
    let listener = &persisted_ingress["HttpListener"];
    assert_eq!(listener["token_type"], serde_json::json!("Bearer"));
    let generated_token = listener["token"]
        .as_str()
        .expect("consumer pull listener must mint a bearer token");
    assert_eq!(generated_token.len(), 64);
    assert!(!cfg_reason.contains(generated_token));

    assert_eq!(entries[1].0, Some(TransferState::Auth));
    assert!(entries[1].1.as_deref().unwrap().contains("Authentication completed"));

    assert_eq!(entries[2].0, Some(TransferState::Ready));
    assert!(entries[2].1.as_deref().unwrap().contains("Dataplane ready"));

    assert_eq!(entries[3].0, Some(TransferState::Started));
    assert!(entries[3].1.as_deref().unwrap().contains("Transfer started (handler=ConsumerPull"));

    assert_eq!(entries[4].0, Some(TransferState::Stopped));
    assert!(entries[4].1.as_deref().unwrap().contains(term_reason));

    assert_eq!(entries[5], (Some(TransferState::Terminated), Some(term_reason.to_string())));
}
