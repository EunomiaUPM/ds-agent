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
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
    .execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
        transfer_dto_urn: tp_id,
    }))
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
        .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
            transfer_dto_urn: tp_id.clone(),
        }))
        .await
        .is_err());

    assert!(mgr
        .execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
            transfer_dto_urn: tp_id,
        }))
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
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
        transfer_dto_urn: Urn::from_str("urn:transfer-process:99").unwrap(),
    }))
    .await;

    assert!(result.is_err());
}
