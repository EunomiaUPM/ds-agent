use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneCommand, DataplaneCommandResponse,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_handlers_strategy::DataplaneStrategyFactory;
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct DataplaneManagerRef {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}

impl DataplaneManagerRef {
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        config: Arc<TransferConfig>,
    ) -> Self {
        Self {
            dataplane_entity,
            connector_entity,
            config,
        }
    }

    pub async fn execute_command(
        &self,
        command: DataplaneCommand,
    ) -> Outcome<DataplaneCommandResponse> {
        // SetInit creates a brand-new context — handle it before the continuation path
        if let DataplaneCommand::SetInit(init) = command {
            DataplaneContext::from_init(
                self.dataplane_entity.clone(),
                self.connector_entity.clone(),
                self.config.clone(),
                init,
            )
            .await?;
            return Ok(DataplaneCommandResponse::Ok);
        }

        // All remaining commands carry a continuation that identifies the existing process
        let context = match command.clone() {
            DataplaneCommand::SetStarted(continuation)
            | DataplaneCommand::SetSubscribing(continuation)
            | DataplaneCommand::SetUnsubscribing(continuation)
            | DataplaneCommand::SetStopped(continuation)
            | DataplaneCommand::SetTerminating(continuation) => {
                DataplaneContext::from_continuation(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    continuation,
                )
            }
            cmd => {
                return Ok(DataplaneCommandResponse::Err(
                    Errors::crazy(
                        format!("Dataplane command {} not expected here", cmd),
                        None,
                    )
                    .into(),
                ))
            }
        }
        .await?;

        // Select strategy based on (role, interaction_mode) from the loaded context
        let handler_strategy = DataplaneStrategyFactory::new(
            self.dataplane_entity.clone(),
            self.connector_entity.clone(),
            self.config.clone(),
        )
        .get_strategy(&context);

        // Dispatch to the appropriate handler method
        match command {
            DataplaneCommand::SetStarted(_) => { handler_strategy.set_started(context).await?; }
            DataplaneCommand::SetSubscribing(_) => { handler_strategy.set_subscribing(context).await?; }
            DataplaneCommand::SetUnsubscribing(_) => { handler_strategy.set_unsubscribing(context).await?; }
            DataplaneCommand::SetStopped(_) => { handler_strategy.set_stopped(context).await?; }
            DataplaneCommand::SetTerminating(_) => { handler_strategy.set_terminating(context).await?; }
            _ => unreachable!(),
        };

        Ok(DataplaneCommandResponse::Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::DataplaneManagerRef;
    use crate::data::entities::dataplane_transfers;
    use crate::entities::dataplane_manager_ref::dataplane_commands::{
        DataplaneCommand, DataplaneCommandResponse, DataplaneContinuation,
        DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::entities::dataplane_transfers::{DataplaneTransferDto, InteractionMode, MockDataplaneTransfersEntitiesTrait, NewDataplaneTransferDto, TransferRole, TransferState};
    use crate::{DataplaneAddress, DataplaneTransfersEntitiesTrait};
    use connector::{AuthenticationConfig, ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto, ConnectorMetadata, HttpSpec, InteractionConfig, ProtocolSpec, PullLifecycle, PushLifecycle, TemplateVecString};
    use mockall::mock;
    use std::str::FromStr;
    use std::sync::Arc;
    use urn::Urn;
    use ymir::errors::Outcome;

    mock! {
        pub ConnectorMock {}
        #[async_trait::async_trait]
        impl ConnectorInstanceTrait for ConnectorMock {
            async fn get_instance_by_id(&self, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn get_instance_by_distribution(&self, distribution_id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn upsert_instance(&self, dto: &mut ConnectorInstantiationDto) -> Outcome<ConnectorInstanceDto>;
            async fn delete_instance_by_id(&self, id: &Urn) -> Outcome<()>;
        }
    }

    fn dummy_config() -> Arc<common::config::services::TransferConfig> {
        let json = serde_json::json!({
            "common": {
                "hosts": {
                    "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null },
                    "grpc": null,
                    "graphql": null
                },
                "db": { "db_type": "Postgres", "url": "localhost", "port": "5432" },
                "api": { "version": "v1", "openapi_path": "/openapi.json" },
                "connection": { "is_local": true, "is_prod": false, "is_vault_real": false, "has_tls_proxy": false }
            },
            "cache": { "cache_type": "Noop", "url": "", "port": "", "user": "", "password": "" },
            "contracts": {
                "hosts": {
                    "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null },
                    "grpc": null, "graphql": null
                },
                "api_version": "v1"
            },
            "catalog": {
                "hosts": {
                    "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null },
                    "grpc": null, "graphql": null
                },
                "api_version": "v1"
            },
            "is_catalog_datahub": false,
            "ssi_auth": {
                "hosts": {
                    "http": { "protocol": "http", "url": "localhost", "port": null, "internal_port": null },
                    "grpc": null, "graphql": null
                },
                "api_version": "v1"
            }
        });
        Arc::new(serde_json::from_value(json).unwrap())
    }

    fn dummy_dataplane_transfer_dto(
        id: &str,
        tp_id: &str,
        role: TransferRole,
        mode: InteractionMode,
        state: TransferState,
        connector_instance_id: Option<Urn>,
    ) -> DataplaneTransferDto {
        DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                id: id.to_string(),
                transfer_process_id: tp_id.to_string(),
                role,
                interaction_mode: mode,
                state,
                connector_instance_id: connector_instance_id.map(|u| u.to_string()),
                ingress_config: serde_json::json!("NoOp"),
                egress_config: serde_json::json!("NoOp"),
                flow_control: None,
                created_at: chrono::Utc::now().into(),
                updated_at: None,
            },
            fields: Default::default(),
            logs: vec![],
        }
    }

    fn dummy_pull_connector(urn: &Urn) -> ConnectorInstanceDto {
        ConnectorInstanceDto {
            id: urn.clone(),
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
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
        }
    }

    fn dummy_push_connector(urn: &Urn) -> ConnectorInstanceDto {
        ConnectorInstanceDto {
            id: urn.clone(),
            metadata: ConnectorMetadata {
                name: None,
                author: None,
                description: None,
                version: None,
                created_at: None,
            },
            authentication_config: AuthenticationConfig::NoAuth,
            interaction: InteractionConfig::Push(PushLifecycle {
                subscribe: ProtocolSpec::Http(HttpSpec {
                    url_template: "http://example.com/data".to_string(),
                    method: TemplateVecString::Value(vec!["GET".to_string()]),
                    headers: None,
                    body_template: None,
                }),
                unsubscribe: Some(ProtocolSpec::Http(HttpSpec {
                    url_template: "http://example.com/data".to_string(),
                    method: TemplateVecString::Value(vec!["GET".to_string()]),
                    headers: None,
                    body_template: None,
                }))
            }),
            distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
        }
    }

    // ─── SetInit ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_init_consumer_pull() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let transfer_process_id = Urn::from_str("urn:transfer-process:1").unwrap();
        let transfer_process_str = transfer_process_id.to_string();

        mock_entity
            .expect_create_dataplane_transfer()
            .withf(move |dto: &NewDataplaneTransferDto| {
                dto.role == TransferRole::Consumer
                    && dto.interaction_mode == InteractionMode::Pull
                    && dto.state == TransferState::Init
                    && dto.connector_instance_id.is_none()
                    && dto.transfer_process_id == transfer_process_str
            })
            .times(1)
            .returning(|dto| {
                Ok(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:1",
                    &dto.transfer_process_id,
                    dto.role.clone(),
                    dto.interaction_mode.clone(),
                    dto.state.clone(),
                    None,
                ))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsConsumer {
                transfer_process_id,
                direction: DataplaneInitCommandDirection::Pull,
            }))
            .await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DataplaneCommandResponse::Ok));
    }

    #[tokio::test]
    async fn test_set_init_consumer_push() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:2").unwrap();

        mock_entity
            .expect_create_dataplane_transfer()
            .withf(|dto: &NewDataplaneTransferDto| {
                dto.role == TransferRole::Consumer && dto.interaction_mode == InteractionMode::Push
            })
            .times(1)
            .returning(|dto| {
                Ok(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:2",
                    &dto.transfer_process_id,
                    dto.role.clone(),
                    dto.interaction_mode.clone(),
                    dto.state.clone(),
                    None,
                ))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsConsumer {
                transfer_process_id: tp_id,
                direction: DataplaneInitCommandDirection::Push {
                    data_address: DataplaneAddress {
                        endpoint_type: "HttpData".to_string(),
                        endpoint: "http://example.com/data".to_string(),
                        authorization_type: None,
                        authorization: None,
                    },
                },
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_init_provider_pull() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:3").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:1").unwrap();
        let connector = dummy_pull_connector(&connector_urn);

        mock_entity
            .expect_create_dataplane_transfer()
            .withf(|dto: &NewDataplaneTransferDto| {
                dto.role == TransferRole::Provider
                    && dto.interaction_mode == InteractionMode::Pull
                    && dto.connector_instance_id.is_some()
            })
            .times(1)
            .returning(|dto| {
                Ok(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:3",
                    &dto.transfer_process_id,
                    dto.role.clone(),
                    dto.interaction_mode.clone(),
                    dto.state.clone(),
                    dto.connector_instance_id.clone(),
                ))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetInit(DataplaneInitCommandTypes::AsProvider {
                transfer_process_id: tp_id,
                connector_instance: connector,
                direction: DataplaneInitCommandDirection::Pull,
            }))
            .await;

        assert!(result.is_ok());
    }

    // ─── Continuation commands ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_started() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:10").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:10",
                    "urn:transfer-process:10",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    TransferState::Ready,
                    None,
                )))
            });

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_, dto| dto.state == Some(TransferState::Started))
            .times(1)
            .returning(|id, dto| {
                Ok(dummy_dataplane_transfer_dto(
                    &id.to_string(),
                    "urn:transfer-process:10",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    dto.state.clone().unwrap(),
                    None,
                ))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
                transfer_dto_urn: tp_id,
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_stopped() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:11").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:11",
                    "urn:transfer-process:11",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    TransferState::Started,
                    None,
                )))
            });

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_, dto| dto.state == Some(TransferState::Stopped))
            .times(1)
            .returning(|id, dto| {
                Ok(dummy_dataplane_transfer_dto(
                    &id.to_string(),
                    "urn:transfer-process:11",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    dto.state.clone().unwrap(),
                    None,
                ))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetStopped(DataplaneContinuation {
                transfer_dto_urn: tp_id,
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_terminating() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:12").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:12",
                    "urn:transfer-process:12",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    TransferState::Started,
                    None,
                )))
            });

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_, dto| dto.state == Some(TransferState::Terminated))
            .times(1)
            .returning(|id, dto| {
                Ok(dummy_dataplane_transfer_dto(
                    &id.to_string(),
                    "urn:transfer-process:12",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    dto.state.clone().unwrap(),
                    None,
                ))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetTerminating(DataplaneContinuation {
                transfer_dto_urn: tp_id,
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_subscribing_noop() {
        // Default set_subscribing in the trait just returns Ok(context) without touching the DB
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:13").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:13",
                    "urn:transfer-process:13",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    TransferState::Ready,
                    None,
                )))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetSubscribing(DataplaneContinuation {
                transfer_dto_urn: tp_id,
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_unsubscribing_noop() {
        // Default set_unsubscribing in the trait just returns Ok(context) without touching the DB
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:14").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:14",
                    "urn:transfer-process:14",
                    TransferRole::Consumer,
                    InteractionMode::Pull,
                    TransferState::Started,
                    None,
                )))
            });

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
                transfer_dto_urn: tp_id,
            }))
            .await;

        assert!(result.is_ok());
    }

    // ─── Error paths ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_unexpected_command_returns_err_response() {
        // SetConfiguring has no continuation — manager returns Ok(DataplaneCommandResponse::Err),
        // NOT Outcome::Err, so the caller can handle it gracefully.
        let mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetConfiguring)
            .await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DataplaneCommandResponse::Err(_)));
    }

    #[tokio::test]
    async fn test_continuation_not_found_returns_err() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| Ok(None));

        let result = DataplaneManagerRef::new(Arc::new(mock_entity), Arc::new(mock_connector), dummy_config())
            .execute_command(DataplaneCommand::SetStarted(DataplaneContinuation {
                transfer_dto_urn: Urn::from_str("urn:transfer-process:99").unwrap(),
            }))
            .await;

        assert!(result.is_err());
    }
}
