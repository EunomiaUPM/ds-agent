#[cfg(test)]
mod tests {
    use crate::entities::dataplane_manager::dataplane_manager::DataplaneManager;
    use crate::entities::dataplane_manager::driver_factory::DataplaneDriverFactory;
    use crate::entities::dataplane_manager::{DataplaneCommand, DataplaneManagerInput};
    use crate::entities::dataplane_transfers::MockDataplaneTransfersEntitiesTrait;
    use crate::entities::dataplane_transfers::{InteractionMode, TransferRole, TransferState, DataplaneTransferDto, NewDataplaneTransferDto, EditDataplaneTransferDto};
    use rainbow_connector::{ConnectorInstanceDto, InteractionConfig, PullLifecycle, PushLifecycle, AuthenticationConfig, ConnectorMetadata, ProtocolSpec, HttpSpec, TemplateVecString, ConnectorInstanceTrait};
    use rainbow_common::config::types::roles::RoleConfig;
    use std::sync::Arc;
    use std::str::FromStr;
    use urn::Urn;
    use crate::data::entities::dataplane_transfers;
    use mockall::mock;

    mock! {
        pub ConnectorInstance {}
        #[async_trait::async_trait]
        impl ConnectorInstanceTrait for ConnectorInstance {
            async fn get_instance_by_id(&self, id: &Urn) -> anyhow::Result<Option<ConnectorInstanceDto>>;
            async fn get_instance_by_distribution(&self, distribution_id: &Urn) -> anyhow::Result<Option<ConnectorInstanceDto>>;
            async fn upsert_instance(&self, instance_dto: &mut rainbow_connector::ConnectorInstantiationDto) -> anyhow::Result<ConnectorInstanceDto>;
            async fn delete_instance_by_id(&self, id: &Urn) -> anyhow::Result<()>;
        }
    }

    fn create_dummy_metadata() -> ConnectorMetadata {
        ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        }
    }

    fn create_dummy_dto(id: &str, tp_id: &str, role: TransferRole, mode: InteractionMode, state: TransferState, connector_instance_id: Option<Urn>) -> DataplaneTransferDto {
        DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                id: id.to_string(),
                transfer_process_id: tp_id.to_string(),
                role,
                interaction_mode: mode,
                state,
                connector_instance_id: connector_instance_id.map(|u| u.to_string()),
                ingress_config: serde_json::Value::Null,
                egress_config: serde_json::Value::Null,
                flow_control: None,
                created_at: chrono::Utc::now().into(),
                updated_at: None,
            },
            fields: Default::default(),
            logs: vec![],
        }
    }

    #[tokio::test]
    async fn test_provision_new_process_consumer_pull() {
        let mut mock_dataplane_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector_entity = MockConnectorInstance::new();
        let driver_factory = Arc::new(DataplaneDriverFactory {});

        let transfer_process_id = Urn::from_str("urn:transfer-process:1").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:1").unwrap();

        mock_dataplane_entity
            .expect_get_dataplane_transfer_by_process_id()
            .with(mockall::predicate::eq(transfer_process_id.clone()))
            .times(1)
            .returning(|_| Ok(None));

        let connector_instance = ConnectorInstanceDto {
            id: connector_urn.clone(),
            metadata: create_dummy_metadata(),
            authentication_config: AuthenticationConfig::NoAuth,
            interaction: InteractionConfig::Pull(PullLifecycle {
                data_access: ProtocolSpec::Http(HttpSpec {
                    url_template: "http://example.com/data".to_string(),
                    method: TemplateVecString::Value(vec!["GET".to_string()]),
                    headers: None,
                    body_template: None,
                }),
            }),
            distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
        };

        mock_connector_entity
            .expect_get_instance_by_id()
            .with(mockall::predicate::eq(connector_urn.clone()))
            .times(1)
            .returning(move |_| Ok(Some(connector_instance.clone())));

        mock_dataplane_entity
            .expect_create_dataplane_transfer()
            .withf(move |dto: &NewDataplaneTransferDto| {
                dto.role == TransferRole::Consumer && 
                dto.interaction_mode == InteractionMode::Pull &&
                dto.state == TransferState::Init
            })
            .times(1)
            .returning(move |dto| {
                Ok(create_dummy_dto("urn:dataplane:1", &dto.transfer_process_id, dto.role.clone(), dto.interaction_mode.clone(), dto.state.clone(), dto.connector_instance_id.clone()))
            });

        let manager = DataplaneManager::new(
            Arc::new(mock_dataplane_entity),
            Arc::new(mock_connector_entity),
            driver_factory,
        );

        let input = DataplaneManagerInput {
            transfer_process_id,
            command: DataplaneCommand::SetInit {
                role: RoleConfig::Consumer,
                connector_instance: Some(connector_urn),
                data_address: None,
            },
        };

        let result = manager.execute_command(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provision_new_process_provider_push() {
        let mut mock_dataplane_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector_entity = MockConnectorInstance::new();
        let driver_factory = Arc::new(DataplaneDriverFactory {});

        let transfer_process_id = Urn::from_str("urn:transfer-process:2").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:2").unwrap();

        mock_dataplane_entity
            .expect_get_dataplane_transfer_by_process_id()
            .with(mockall::predicate::eq(transfer_process_id.clone()))
            .times(1)
            .returning(|_| Ok(None));

        let connector_instance = ConnectorInstanceDto {
            id: connector_urn.clone(),
            metadata: create_dummy_metadata(),
            authentication_config: AuthenticationConfig::NoAuth,
            interaction: InteractionConfig::Push(PushLifecycle {
                subscribe: ProtocolSpec::Http(HttpSpec {
                    url_template: "http://example.com/push".to_string(),
                    method: TemplateVecString::Value(vec!["POST".to_string()]),
                    headers: None,
                    body_template: None,
                }),
                unsubscribe: None,
            }),
            distribution_id: Urn::from_str("urn:distribution:2").unwrap(),
        };

        mock_connector_entity
            .expect_get_instance_by_id()
            .with(mockall::predicate::eq(connector_urn.clone()))
            .times(1)
            .returning(move |_| Ok(Some(connector_instance.clone())));

        mock_dataplane_entity
            .expect_create_dataplane_transfer()
            .withf(move |dto: &NewDataplaneTransferDto| {
                dto.role == TransferRole::Provider && 
                dto.interaction_mode == InteractionMode::Push &&
                dto.state == TransferState::Init
            })
            .times(1)
            .returning(move |dto| {
                Ok(create_dummy_dto("urn:dataplane:2", &dto.transfer_process_id, dto.role.clone(), dto.interaction_mode.clone(), dto.state.clone(), dto.connector_instance_id.clone()))
            });

        let manager = DataplaneManager::new(
            Arc::new(mock_dataplane_entity),
            Arc::new(mock_connector_entity),
            driver_factory,
        );

        let input = DataplaneManagerInput {
            transfer_process_id,
            command: DataplaneCommand::SetInit {
                role: RoleConfig::Provider,
                connector_instance: Some(connector_urn),
                data_address: None,
            },
        };

        let result = manager.execute_command(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pull_consumer_init_to_configuring() {
        let mut mock_dataplane_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector_entity = MockConnectorInstance::new();
        let driver_factory = Arc::new(DataplaneDriverFactory {});

        let transfer_process_id = Urn::from_str("urn:transfer-process:3").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:1").unwrap();

        let existing_process = create_dummy_dto("urn:dataplane:3", &transfer_process_id.to_string(), TransferRole::Consumer, InteractionMode::Pull, TransferState::Init, Some(connector_urn.clone()));

        mock_dataplane_entity
            .expect_get_dataplane_transfer_by_process_id()
            .with(mockall::predicate::eq(transfer_process_id.clone()))
            .times(1)
            .returning(move |_| Ok(Some(existing_process.clone())));

        let connector_instance = ConnectorInstanceDto {
            id: connector_urn.clone(),
            metadata: create_dummy_metadata(),
            authentication_config: AuthenticationConfig::NoAuth,
            interaction: InteractionConfig::Pull(PullLifecycle {
                data_access: ProtocolSpec::Http(HttpSpec {
                    url_template: "http://example.com/data".to_string(),
                    method: TemplateVecString::Value(vec!["GET".to_string()]),
                    headers: None,
                    body_template: None,
                }),
            }),
            distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
        };

        mock_connector_entity
            .expect_get_instance_by_id()
            .with(mockall::predicate::eq(connector_urn.clone()))
            .times(1)
            .returning(move |_| Ok(Some(connector_instance.clone())));

        mock_dataplane_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_id, dto: &EditDataplaneTransferDto| {
                dto.state == Some(TransferState::Configuring)
            })
            .times(1)
            .returning(move |_id, _dto| Ok(create_dummy_dto("urn:dataplane:3", "urn:transfer-process:3", TransferRole::Consumer, InteractionMode::Pull, TransferState::Configuring, Some(Urn::from_str("urn:connector-instance:1").unwrap()))));

        let manager = DataplaneManager::new(
            Arc::new(mock_dataplane_entity),
            Arc::new(mock_connector_entity),
            driver_factory,
        );

        let input = DataplaneManagerInput {
            transfer_process_id,
            command: DataplaneCommand::SetInit {
                role: RoleConfig::Consumer,
                connector_instance: Some(connector_urn),
                data_address: None,
            },
        };

        let result = manager.execute_command(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pull_consumer_ready_to_started() {}
    #[tokio::test]
    async fn test_pull_provider_init_to_configuring() {}
    #[tokio::test]
    async fn test_pull_provider_ready_to_started() {}
    #[tokio::test]
    async fn test_push_consumer_init_to_configuring() {}
    #[tokio::test]
    async fn test_push_consumer_starting_to_started() {}
    #[tokio::test]
    async fn test_push_provider_init_to_configuring() {}
    #[tokio::test]
    async fn test_push_provider_ready_to_subscribing() {}
    #[tokio::test]
    async fn test_transition_configuring_to_auth() {}
    #[tokio::test]
    async fn test_transition_auth_to_ready() {}
    #[tokio::test]
    async fn test_transition_any_to_terminated() {}
}
