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

//! Continuations on provider pull transfers: state changes, no subscriber.

use super::*;

/// SetStarted on a provider pull transfer persists Started.
#[tokio::test]
async fn set_started_provider_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();
    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

    let tp_id = Urn::from_str("urn:transfer-process:20").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:20").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:20",
                "urn:transfer-process:20",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Ready,
                Some(Urn::from_str("urn:connector-instance:20").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_pull_connector(&connector_urn))));

    // mock_factory is used by from_continuation; set_configuring uses the real
    // DataplaneDriverFactory
    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(|_| Ok(dummy_driver()));

    mock_entity
        .expect_edit()
        .withf(|_, _, dto| dto.state == Some(TransferState::Started))
        .times(1)
        .returning(|_, id, dto| {
            Ok(dummy_dataplane_transfer_dto(
                id.as_ref(),
                "urn:transfer-process:20",
                TransferRole::Provider,
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
    .with_driver_factory(Arc::new(mock_factory))
    .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
        transfer_dto_urn: tp_id,
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}

/// SetStopped on a provider pull transfer persists Stopped.
#[tokio::test]
async fn set_stopped_provider_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();
    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

    let tp_id = Urn::from_str("urn:transfer-process:21").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:21").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:21",
                "urn:transfer-process:21",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Started,
                Some(Urn::from_str("urn:connector-instance:21").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_pull_connector(&connector_urn))));

    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(|_| Ok(dummy_driver()));

    mock_entity
        .expect_edit()
        .withf(|_, _, dto| dto.state == Some(TransferState::Stopped))
        .times(1)
        .returning(|_, id, dto| {
            Ok(dummy_dataplane_transfer_dto(
                id.as_ref(),
                "urn:transfer-process:21",
                TransferRole::Provider,
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
    .with_driver_factory(Arc::new(mock_factory))
    .execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
        transfer_dto_urn: tp_id,
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}

/// SetTerminating on a provider pull transfer persists Terminated.
#[tokio::test]
async fn set_terminating_provider_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();
    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

    let tp_id = Urn::from_str("urn:transfer-process:22").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:22").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:22",
                "urn:transfer-process:22",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Started,
                Some(Urn::from_str("urn:connector-instance:22").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_pull_connector(&connector_urn))));

    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(|_| Ok(dummy_driver()));

    mock_entity
        .expect_edit()
        .withf(|_, _, dto| dto.state == Some(TransferState::Terminated))
        .times(1)
        .returning(|_, id, dto| {
            Ok(dummy_dataplane_transfer_dto(
                id.as_ref(),
                "urn:transfer-process:22",
                TransferRole::Provider,
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
    .with_driver_factory(Arc::new(mock_factory))
    .execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
        transfer_dto_urn: tp_id,
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}

/// SetSubscribing on a provider pull transfer persists nothing.
#[tokio::test]
async fn set_subscribing_noop_provider_pull() {
    // Pull providers have no subscriber - set_subscribing is noop
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();
    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

    let tp_id = Urn::from_str("urn:transfer-process:23").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:23").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:23",
                "urn:transfer-process:23",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Ready,
                Some(Urn::from_str("urn:connector-instance:23").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_pull_connector(&connector_urn))));

    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(|_| Ok(dummy_driver_pull()));

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .with_driver_factory(Arc::new(mock_factory))
    .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
        transfer_dto_urn: tp_id,
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}

/// SetUnsubscribing on a provider pull transfer persists nothing.
#[tokio::test]
async fn set_unsubscribing_noop_provider_pull() {
    // Pull providers have no subscriber - set_unsubscribing is noop
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();
    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

    let tp_id = Urn::from_str("urn:transfer-process:24").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:24").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:24",
                "urn:transfer-process:24",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Started,
                Some(Urn::from_str("urn:connector-instance:24").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_pull_connector(&connector_urn))));

    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(|_| Ok(dummy_driver_pull()));

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .with_driver_factory(Arc::new(mock_factory))
    .execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
        transfer_dto_urn: tp_id,
        tenant_id: "tenant-1".to_string(),
    }))
    .await;

    assert!(result.is_ok());
}
