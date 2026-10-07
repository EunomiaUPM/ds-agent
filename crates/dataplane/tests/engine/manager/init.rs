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

//! SetInit per role and direction: the transfer is created and walked to its first stable state.

use super::*;

/// A consumer pull init creates the transfer in Init and answers with the proxy address.
#[tokio::test]
async fn set_init_consumer_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let transfer_process_id = Urn::from_str("urn:transfer-process:1").unwrap();
    let transfer_process_str = transfer_process_id.to_string();

    mock_entity
        .expect_create()
        .withf(move |_, dto: &NewDataplaneTransferDto| {
            dto.role == TransferRole::Consumer
                && dto.interaction_mode == InteractionMode::Pull
                && dto.state == TransferState::Init
                && dto.connector_instance_id.is_none()
                && dto.transfer_process_id == transfer_process_str
        })
        .times(1)
        .returning(|_, dto| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:1",
                &dto.transfer_process_id,
                dto.role.clone(),
                dto.interaction_mode.clone(),
                dto.state.clone(),
                None,
            ))
        });

    // ConsumerPull.set_init is a no-op — no put calls expected

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetInit(
        DataplaneInitCommandTypes::AsConsumer {
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            transfer_process_id,
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(dummy_dataplane_forward_address()),
            },
        },
    ))
    .await;

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        DataplaneCommandResponse::OkWithAddress(_)
    ));
}

/// A consumer push init creates the transfer and walks it through Configuring, Auth and
/// Ready.
#[tokio::test]
async fn set_init_consumer_push() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:2").unwrap();

    mock_entity
        .expect_create()
        .withf(|_, dto: &NewDataplaneTransferDto| {
            dto.role == TransferRole::Consumer && dto.interaction_mode == InteractionMode::Push
        })
        .times(1)
        .returning(|_, dto| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:2",
                &dto.transfer_process_id,
                dto.role.clone(),
                dto.interaction_mode.clone(),
                dto.state.clone(),
                None,
            ))
        });

    for state in [
        TransferState::Configuring,
        TransferState::Auth,
        TransferState::Ready,
    ] {
        let s = state.clone();
        mock_entity
            .expect_edit()
            .times(1)
            .withf(move |_, _, edit| edit.state == Some(s.clone()))
            .returning(move |_, _, _| {
                Ok(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:2",
                    "urn:transfer-process:2",
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
    .execute_command(DataplaneCommand::SetInit(
        DataplaneInitCommandTypes::AsConsumer {
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            transfer_process_id: tp_id,
            direction: DataplaneInitCommandDirection::Push {
                data_address: Some(DataplaneAddress {
                    endpoint_type: "HttpData".to_string(),
                    endpoint: "http://example.com/data".to_string(),
                    authorization_type: None,
                    authorization: None,
                }),
            },
        },
    ))
    .await;

    assert!(result.is_ok());
}

/// A provider pull init creates the transfer with its connector and walks it through
/// Configuring, Auth and Ready.
#[tokio::test]
async fn set_init_provider_pull() {
    let mut mock_entity = MockDataplaneTransferServiceTrait::new();
    let mock_connector = MockConnectorMock::new();

    let tp_id = Urn::from_str("urn:transfer-process:3").unwrap();
    let connector_urn = Urn::from_str("urn:connector-instance:1").unwrap();
    let connector = dummy_pull_connector(&connector_urn);

    mock_entity
        .expect_create()
        .withf(|_, dto: &NewDataplaneTransferDto| {
            dto.role == TransferRole::Provider
                && dto.interaction_mode == InteractionMode::Pull
                && dto.connector_instance_id.is_some()
        })
        .times(1)
        .returning(|_, dto| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:3",
                &dto.transfer_process_id,
                dto.role.clone(),
                dto.interaction_mode.clone(),
                dto.state.clone(),
                dto.connector_instance_id.clone(),
            ))
        });

    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Configuring))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:3",
                "urn:transfer-process:3",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Configuring,
                Some(Urn::from_str("urn:connector-instance:1").unwrap()),
            ))
        });

    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Auth))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:3",
                "urn:transfer-process:3",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Auth,
                Some(Urn::from_str("urn:connector-instance:1").unwrap()),
            ))
        });

    mock_entity
        .expect_edit()
        .times(1)
        .withf(|_, _, edit| edit.state == Some(TransferState::Ready))
        .returning(|_, _, _| {
            Ok(dummy_dataplane_transfer_dto(
                "urn:dataplane-transfer:3",
                "urn:transfer-process:3",
                TransferRole::Provider,
                InteractionMode::Pull,
                TransferState::Ready,
                Some(Urn::from_str("urn:connector-instance:1").unwrap()),
            ))
        });

    let result = DataplaneManager::new(
        Arc::new(mock_entity),
        Arc::new(mock_connector),
        transfer_config_fixture(),
    )
    .execute_command(DataplaneCommand::SetInit(
        DataplaneInitCommandTypes::AsProvider {
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            transfer_process_id: tp_id,
            connector_instance: connector,
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(dummy_dataplane_forward_address()),
            },
        },
    ))
    .await;

    assert!(result.is_ok());
}
