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

/// SetStarted loads the transfer by process id and persists Started.
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
                TransferState::Ready,
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
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
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
        tenant_id: "tenant-1".to_string(),
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
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}

/// Default set_subscribing in the trait just returns Ok(context) without touching the DB
#[tokio::test]
async fn set_subscribing_noop() {
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
                TransferState::Ready,
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
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}

/// Default set_unsubscribing in the trait just returns Ok(context) without touching the DB
#[tokio::test]
async fn set_unsubscribing_noop() {
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
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
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
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_err());
}
