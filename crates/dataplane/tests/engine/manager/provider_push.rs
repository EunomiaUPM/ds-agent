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

//! Continuations on provider push transfers: subscribing goes through the driver.

use super::*;

/// Push provider has a subscriber - set_subscribing calls NoOpPubSubscriber.subscribe
#[tokio::test]
async fn set_subscribing_provider_push() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:25").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:25").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:25",
                "urn:transfer-process:25",
                TransferRole::Provider,
                InteractionMode::Push,
                TransferState::Ready,
                Some(Urn::from_str("urn:connector-instance:25").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_push_connector(&connector_urn))));

    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();
    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(move |_| Ok(dummy_driver()));

    // set_subscribing with subscriber: put(Subscribing) - subscribe - set_started -
    // put(Started)
    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Subscribing))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:25",
                "urn:transfer-process:25",
                TransferRole::Provider,
                InteractionMode::Push,
                TransferState::Subscribing,
                Some(Urn::from_str("urn:connector-instance:25").unwrap()),
            ))
        });

    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Started))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:25",
                "urn:transfer-process:25",
                TransferRole::Provider,
                InteractionMode::Push,
                TransferState::Started,
                Some(Urn::from_str("urn:connector-instance:25").unwrap()),
            ))
        });

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

/// Push provider has a subscriber - set_unsubscribing calls NoOpPubSubscriber.unsubscribe
#[tokio::test]
async fn set_unsubscribing_provider_push() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mut mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:26").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:26").unwrap();

    mock_entity
        .expect_get_by_process_id()
        .times(1)
        .returning(|_, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:26",
                "urn:transfer-process:26",
                TransferRole::Provider,
                InteractionMode::Push,
                TransferState::Started,
                Some(Urn::from_str("urn:connector-instance:26").unwrap()),
            ))
        });

    mock_connector
        .expect_get_instance_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(dummy_push_connector(&connector_urn))));

    let mut mock_factory = MockDataplaneDriverFactoryTrait::new();
    mock_factory
        .expect_get_or_create_driver()
        .times(1)
        .returning(move |_| Ok(dummy_driver()));

    // set_unsubscribing with subscriber: put(Unsubscribing) - unsubscribe - set_stopped -
    // put(Stopped)
    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Unsubscribing))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:26",
                "urn:transfer-process:26",
                TransferRole::Provider,
                InteractionMode::Push,
                TransferState::Unsubscribing,
                Some(Urn::from_str("urn:connector-instance:26").unwrap()),
            ))
        });

    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Stopped))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:26",
                "urn:transfer-process:26",
                TransferRole::Provider,
                InteractionMode::Push,
                TransferState::Stopped,
                Some(Urn::from_str("urn:connector-instance:26").unwrap()),
            ))
        });

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
