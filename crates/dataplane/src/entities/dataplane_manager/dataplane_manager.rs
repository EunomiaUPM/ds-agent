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

use crate::entities::dataplane_manager::dataplane_commands::{
    DataplaneCommand, DataplaneCommandResponse,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_driver_factory::{
    DataplaneDriverFactory, DataplaneDriverFactoryTrait,
};
use crate::entities::dataplane_manager::dataplane_handlers_strategy::DataplaneStrategyFactory;
use crate::errors::DataplaneError;
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneManager {
    dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
    driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneManager {
    /// Required dependencies. The driver factory defaults to a keystore-less
    /// `DataplaneDriverFactory` and the secret store to `None`; override either
    /// with the `with_*` methods below.
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        config: Arc<TransferConfig>,
    ) -> Self {
        Self {
            dataplane_entity,
            connector_entity,
            config,
            driver_factory: Arc::new(DataplaneDriverFactory::new()),
            secret_store: None,
        }
    }

    /// Overrides the default driver factory (e.g. one backed by the keystore).
    pub fn with_driver_factory(
        mut self,
        driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
    ) -> Self {
        self.driver_factory = driver_factory;
        self
    }

    /// Enables runtime-secret resolution via the given secret store.
    pub fn with_secret_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.secret_store = Some(store);
        self
    }

    pub async fn execute_command(
        &self,
        command: DataplaneCommand,
    ) -> Outcome<DataplaneCommandResponse> {
        let mut context = match &command {
            DataplaneCommand::SetInit(init) => {
                DataplaneContext::from_init(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    init.clone(),
                )
                .await?
            }
            DataplaneCommand::SetConfiguring((cont, address)) => {
                DataplaneContext::from_continuation(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.driver_factory.clone(),
                    self.config.clone(),
                    cont.clone(),
                    Some(address.clone()),
                )
                .await?
            }
            DataplaneCommand::GetAssociated(continuation)
            | DataplaneCommand::SetStarted(continuation)
            | DataplaneCommand::SetSubscribing(continuation)
            | DataplaneCommand::SetUnsubscribing(continuation)
            | DataplaneCommand::SetStopped(continuation)
            | DataplaneCommand::SetTerminating(continuation) => {
                DataplaneContext::from_continuation(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.driver_factory.clone(),
                    self.config.clone(),
                    continuation.clone(),
                    None,
                )
                .await?
            }
            cmd => {
                return Err(DataplaneError::UnexpectedCommand {
                    command: cmd.to_string(),
                }
                .into())
            }
        };

        // Resolve runtime secret placeholders before dispatch.
        if let (Some(runtime), Some(store)) = (context.runtime().cloned(), &self.secret_store) {
            use crate::entities::dataplane_manager::dataplane_runtime::RuntimeSecretVault;
            let resolved = RuntimeSecretVault::new(store.as_ref())
                .resolve(runtime)
                .await;
            context.set_runtime(resolved);
        }

        // Select strategy based on (role, interaction_mode) from the loaded context
        let handler_strategy = DataplaneStrategyFactory::new(
            self.dataplane_entity.clone(),
            self.connector_entity.clone(),
            self.config.clone(),
            self.secret_store.clone(),
        )
        .get_strategy(&context);

        // Dispatch to the appropriate handler method
        let new_context = match command {
            DataplaneCommand::GetAssociated(_) => handler_strategy.get_associated(context),
            DataplaneCommand::SetInit(_) => handler_strategy.set_init(context),
            DataplaneCommand::SetConfiguring(_) => handler_strategy.set_configuring(context),
            DataplaneCommand::SetStarted(_) => handler_strategy.set_started(context),
            DataplaneCommand::SetSubscribing(_) => handler_strategy.set_subscribing(context),
            DataplaneCommand::SetUnsubscribing(_) => handler_strategy.set_unsubscribing(context),
            DataplaneCommand::SetStopped(_) => handler_strategy.set_stopped(context),
            DataplaneCommand::SetTerminating(_) => handler_strategy.set_terminating(context),
            cmd => unreachable!("command '{}' was rejected in context-building phase", cmd),
        }
        .await?;

        if let Some(forward_address) = new_context.forward_dataplane_address() {
            Ok(DataplaneCommandResponse::OkWithAddress(
                forward_address.clone(),
            ))
        } else {
            Ok(DataplaneCommandResponse::Ok)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DataplaneManager;
    use crate::data::entities::dataplane_transfers;
    use crate::entities::dataplane_drivers::authentication::no_op::NoOpAuthenticator;
    use crate::entities::dataplane_drivers::configuration::no_op::NoOpProxyConfigurator;
    use crate::entities::dataplane_drivers::pubsub::no_op::NoOpPubSubscriber;
    use crate::entities::dataplane_drivers::{
        DataplaneDriver, DriverAuthenticatorTrait, DriverProxyConfiguratorTrait, DriverPubSubTrait,
        MockDriverPubSubTrait,
    };
    use crate::entities::dataplane_manager::dataplane_commands::{
        DataplaneCommand, DataplaneCommandResponse, DataplaneContinuation,
        DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
    use crate::entities::dataplane_manager::dataplane_driver_factory::{
        DataplaneDriverFactoryTrait, MockDataplaneDriverFactoryTrait,
    };
    use crate::entities::dataplane_transfers::{
        DataplaneTransferDto, InteractionMode, MockDataplaneTransfersEntitiesTrait,
        NewDataplaneTransferDto, TransferRole, TransferState,
    };
    use crate::{DataplaneAddress, DataplaneTransfersEntitiesTrait};
    use common::test_utils::config_fixtures::transfer_config_fixture;
    use connector::{
        AuthenticationConfig, ConnectorInstanceDto, ConnectorInstanceTrait,
        ConnectorInstantiationDto, ConnectorMetadata, HttpSpec, InteractionConfig, ProtocolSpec,
        PullLifecycle, PushLifecycle, TemplateVecString,
    };
    use mockall::mock;
    use std::str::FromStr;
    use std::sync::Arc;
    use urn::Urn;
    use ymir::errors::Outcome as MockOutcome;
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

    fn dummy_driver() -> DataplaneDriver {
        DataplaneDriver {
            authenticator: Arc::new(NoOpAuthenticator),
            proxy_configurator: Arc::new(NoOpProxyConfigurator),
            subscriber: Some(Arc::new(NoOpPubSubscriber)),
        }
    }

    fn dummy_driver_pull() -> DataplaneDriver {
        DataplaneDriver {
            authenticator: Arc::new(NoOpAuthenticator),
            proxy_configurator: Arc::new(NoOpProxyConfigurator),
            subscriber: None,
        }
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
                })),
            }),
            distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
        }
    }

    fn dummy_dataplane_forward_address() -> DataplaneAddress {
        DataplaneAddress {
            endpoint_type: "HTTP".to_string(),
            endpoint: "http://dummy-data-address.com".to_string(),
            authorization_type: Some("Dummy".to_string()),
            authorization: Some("Dummy".to_string()),
        }
    }

    // SetInit ───────────────────────────────────────────────────────────────

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

        // ConsumerPull.set_init is a no-op — no put calls expected

        let result = DataplaneManager::new(
            Arc::new(mock_entity),
            Arc::new(mock_connector),
            transfer_config_fixture(),
        )
        .execute_command(DataplaneCommand::SetInit(
            DataplaneInitCommandTypes::AsConsumer {
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

        for state in [
            TransferState::Configuring,
            TransferState::Auth,
            TransferState::Ready,
        ] {
            let s = state.clone();
            mock_entity
                .expect_put_dataplane_transfer_by_id()
                .times(1)
                .withf(move |_, edit| edit.state == Some(s.clone()))
                .returning(move |_, _| {
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

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Configuring))
            .returning(|_, _| {
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
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Auth))
            .returning(|_, _| {
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
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Ready))
            .returning(|_, _| {
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

    // Continuation commands ─────────────────────────────────────────────────

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

        let result = DataplaneManager::new(
            Arc::new(mock_entity),
            Arc::new(mock_connector),
            transfer_config_fixture(),
        )
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

        let result = DataplaneManager::new(
            Arc::new(mock_entity),
            Arc::new(mock_connector),
            transfer_config_fixture(),
        )
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

        let result = DataplaneManager::new(
            Arc::new(mock_entity),
            Arc::new(mock_connector),
            transfer_config_fixture(),
        )
        .execute_command(DataplaneCommand::SetUnsubscribing(DataplaneContinuation {
            transfer_dto_urn: tp_id,
        }))
        .await;

        assert!(result.is_ok());
    }

    // Error paths ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_unexpected_command_returns_err_response() {
        // SetAuth is an internal state-machine step — never a valid external command.
        let mock_entity = MockDataplaneTransfersEntitiesTrait::new();
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

    #[tokio::test]
    async fn test_continuation_not_found_returns_err() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mock_connector = MockConnectorMock::new();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| Ok(None));

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

    // Provider/Pull ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_started_provider_pull() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();
        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

        let tp_id = Urn::from_str("urn:transfer-process:20").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:20").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:20",
                    "urn:transfer-process:20",
                    TransferRole::Provider,
                    InteractionMode::Pull,
                    TransferState::Ready,
                    Some(Urn::from_str("urn:connector-instance:20").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_pull_connector(&connector_urn))));

        // mock_factory is used by from_continuation; set_configuring uses the real
        // DataplaneDriverFactory
        mock_factory
            .expect_get_or_create_driver()
            .times(1)
            .returning(|_| Ok(dummy_driver()));

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_, dto| dto.state == Some(TransferState::Started))
            .times(1)
            .returning(|id, dto| {
                Ok(dummy_dataplane_transfer_dto(
                    &id.to_string(),
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
        }))
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_stopped_provider_pull() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();
        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

        let tp_id = Urn::from_str("urn:transfer-process:21").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:21").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:21",
                    "urn:transfer-process:21",
                    TransferRole::Provider,
                    InteractionMode::Pull,
                    TransferState::Started,
                    Some(Urn::from_str("urn:connector-instance:21").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_pull_connector(&connector_urn))));

        mock_factory
            .expect_get_or_create_driver()
            .times(1)
            .returning(|_| Ok(dummy_driver()));

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_, dto| dto.state == Some(TransferState::Stopped))
            .times(1)
            .returning(|id, dto| {
                Ok(dummy_dataplane_transfer_dto(
                    &id.to_string(),
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
        }))
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_terminating_provider_pull() {
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();
        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

        let tp_id = Urn::from_str("urn:transfer-process:22").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:22").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:22",
                    "urn:transfer-process:22",
                    TransferRole::Provider,
                    InteractionMode::Pull,
                    TransferState::Started,
                    Some(Urn::from_str("urn:connector-instance:22").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_pull_connector(&connector_urn))));

        mock_factory
            .expect_get_or_create_driver()
            .times(1)
            .returning(|_| Ok(dummy_driver()));

        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .withf(|_, dto| dto.state == Some(TransferState::Terminated))
            .times(1)
            .returning(|id, dto| {
                Ok(dummy_dataplane_transfer_dto(
                    &id.to_string(),
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
        }))
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_subscribing_noop_provider_pull() {
        // Pull providers have no subscriber - set_subscribing is noop
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();
        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

        let tp_id = Urn::from_str("urn:transfer-process:23").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:23").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:23",
                    "urn:transfer-process:23",
                    TransferRole::Provider,
                    InteractionMode::Pull,
                    TransferState::Ready,
                    Some(Urn::from_str("urn:connector-instance:23").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_pull_connector(&connector_urn))));

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
        }))
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_unsubscribing_noop_provider_pull() {
        // Pull providers have no subscriber - set_unsubscribing is noop
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();
        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();

        let tp_id = Urn::from_str("urn:transfer-process:24").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:24").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:24",
                    "urn:transfer-process:24",
                    TransferRole::Provider,
                    InteractionMode::Pull,
                    TransferState::Started,
                    Some(Urn::from_str("urn:connector-instance:24").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_pull_connector(&connector_urn))));

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
        }))
        .await;

        assert!(result.is_ok());
    }

    // Provider/Push ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_subscribing_provider_push() {
        // Push provider has a subscriber - set_subscribing calls NoOpPubSubscriber.subscribe
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:25").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:25").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:25",
                    "urn:transfer-process:25",
                    TransferRole::Provider,
                    InteractionMode::Push,
                    TransferState::Ready,
                    Some(Urn::from_str("urn:connector-instance:25").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_push_connector(&connector_urn))));

        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();
        mock_factory
            .expect_get_or_create_driver()
            .times(1)
            .returning(move |_| Ok(dummy_driver()));

        // set_subscribing with subscriber: put(Subscribing) - subscribe - set_started -
        // put(Started)
        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Subscribing))
            .returning(|_, _| {
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
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Started))
            .returning(|_, _| {
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
        }))
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_unsubscribing_provider_push() {
        // Push provider has a subscriber - set_unsubscribing calls NoOpPubSubscriber.unsubscribe
        let mut mock_entity = MockDataplaneTransfersEntitiesTrait::new();
        let mut mock_connector = MockConnectorMock::new();

        let tp_id = Urn::from_str("urn:transfer-process:26").unwrap();
        let connector_urn = Urn::from_str("urn:connector-instance:26").unwrap();

        mock_entity
            .expect_get_dataplane_transfer_by_process_id()
            .times(1)
            .returning(|_| {
                Ok(Some(dummy_dataplane_transfer_dto(
                    "urn:dataplane-transfer:26",
                    "urn:transfer-process:26",
                    TransferRole::Provider,
                    InteractionMode::Push,
                    TransferState::Started,
                    Some(Urn::from_str("urn:connector-instance:26").unwrap()),
                )))
            });

        mock_connector
            .expect_get_instance_by_id()
            .times(1)
            .returning(move |_| Ok(Some(dummy_push_connector(&connector_urn))));

        let mut mock_factory = MockDataplaneDriverFactoryTrait::new();
        mock_factory
            .expect_get_or_create_driver()
            .times(1)
            .returning(move |_| Ok(dummy_driver()));

        // set_unsubscribing with subscriber: put(Unsubscribing) - unsubscribe - set_stopped -
        // put(Stopped)
        mock_entity
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Unsubscribing))
            .returning(|_, _| {
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
            .expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(|_, edit| edit.state == Some(TransferState::Stopped))
            .returning(|_, _| {
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
        }))
        .await;

        assert!(result.is_ok());
    }
}
