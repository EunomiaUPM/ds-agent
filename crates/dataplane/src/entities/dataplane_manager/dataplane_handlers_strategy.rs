use crate::entities::dataplane_manager::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_handlers_consumer_pull::DataplaneHandlerConsumerPull;
use crate::entities::dataplane_manager::dataplane_handlers_consumer_push::DataplaneHandlerConsumerPush;
use crate::entities::dataplane_manager::dataplane_handlers_provider_pull::DataplaneHandlerProviderPull;
use crate::entities::dataplane_manager::dataplane_handlers_provider_push::DataplaneHandlerProviderPush;
use crate::entities::dataplane_transfers::{InteractionMode, TransferRole};
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneStrategyFactory {
    dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}
impl DataplaneStrategyFactory {
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        config: Arc<TransferConfig>,
        secret_store: Option<Arc<dyn SecretStore>>,
    ) -> Self {
        Self {
            dataplane_entity,
            connector_entity,
            config,
            secret_store,
        }
    }
    pub fn get_strategy(
        &self,
        context: &DataplaneContext,
    ) -> Box<dyn DataplaneCommandStateMachine> {
        let role = context.dataplane_process_role();
        let interaction_mode = context.dataplane_process_interaction_mode();
        match (role, interaction_mode) {
            (TransferRole::Provider, InteractionMode::Pull) => {
                Box::new(DataplaneHandlerProviderPull::new(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
            (TransferRole::Provider, InteractionMode::Push) => {
                Box::new(DataplaneHandlerProviderPush::new(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
            (TransferRole::Consumer, InteractionMode::Pull) => {
                Box::new(DataplaneHandlerConsumerPull::new(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
            (TransferRole::Consumer, InteractionMode::Push) => {
                Box::new(DataplaneHandlerConsumerPush::new(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::entities::dataplane_transfers;
    use crate::entities::dataplane_manager::dataplane_commands::{
        DataplaneCommandStateMachine, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
    use crate::entities::dataplane_manager::dataplane_handlers_strategy::DataplaneStrategyFactory;
    use crate::entities::dataplane_transfers::{
        DataplaneTransferDto, MockDataplaneTransfersEntitiesTrait, TransferState,
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

    // Entity mock that handles the create call made by DataplaneContext::from_init,
    // reflecting back the role and mode from the NewDataplaneTransferDto it receives.
    fn entity_for_init() -> Arc<dyn DataplaneTransfersEntitiesTrait> {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        mock.expect_create_dataplane_transfer().returning(|dto| {
            Ok(DataplaneTransferDto {
                inner: dataplane_transfers::Model {
                    id: "urn:dataplane-transfer:test".to_string(),
                    transfer_process_id: dto.transfer_process_id.clone(),
                    role: dto.role.clone(),
                    interaction_mode: dto.interaction_mode.clone(),
                    state: TransferState::Init,
                    connector_instance_id: dto
                        .connector_instance_id
                        .as_ref()
                        .map(|u| u.to_string()),
                    ingress_config: serde_json::json!("NoOp"),
                    egress_config: serde_json::json!("NoOp"),
                    flow_control: None,
                    created_at: chrono::Utc::now().into(),
                    updated_at: None,
                },
                fields: Default::default(),
                logs: vec![],
            })
        });
        Arc::new(mock)
    }

    async fn dummy_context(init: DataplaneInitCommandTypes) -> DataplaneContext {
        DataplaneContext::from_init(
            entity_for_init(),
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            init,
        )
        .await
        .unwrap()
    }

    // Connector fixture for provider tests — interaction type doesn't affect routing
    // (routing is based on role+direction stored in the DTO, not on connector config).
    fn connector_fixture() -> ConnectorInstanceDto {
        ConnectorInstanceDto {
            id: Urn::from_str("urn:connector-instance:1").unwrap(),
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
                    url_template: "http://example.com/events".to_string(),
                    method: TemplateVecString::Value(vec!["POST".to_string()]),
                    headers: None,
                    body_template: None,
                }),
                unsubscribe: None,
            }),
            distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
        }
    }

    fn get_strategy(context: DataplaneContext) -> Box<dyn DataplaneCommandStateMachine> {
        DataplaneStrategyFactory::new(
            entity_for_init(),
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            None,
        )
        .get_strategy(&context)
    }

    fn empty_address() -> DataplaneAddress {
        DataplaneAddress {
            endpoint_type: "HTTP".to_string(),
            endpoint: "http://example.com".to_string(),
            authorization_type: None,
            authorization: None,
        }
    }

    #[tokio::test]
    async fn test_routes_consumer_pull() {
        let context = dummy_context(DataplaneInitCommandTypes::AsConsumer {
            transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(empty_address()),
            },
        })
        .await;
        assert_eq!(get_strategy(context).handler_name(), "ConsumerPull");
    }

    #[tokio::test]
    async fn test_routes_consumer_push() {
        let context = dummy_context(DataplaneInitCommandTypes::AsConsumer {
            transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
            direction: DataplaneInitCommandDirection::Push {
                data_address: Some(empty_address()),
            },
        })
        .await;
        assert_eq!(get_strategy(context).handler_name(), "ConsumerPush");
    }

    #[tokio::test]
    async fn test_routes_provider_pull() {
        let context = dummy_context(DataplaneInitCommandTypes::AsProvider {
            transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
            connector_instance: connector_fixture(),
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(empty_address()),
            },
        })
        .await;
        assert_eq!(get_strategy(context).handler_name(), "ProviderPull");
    }

    #[tokio::test]
    async fn test_routes_provider_push() {
        let context = dummy_context(DataplaneInitCommandTypes::AsProvider {
            transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
            connector_instance: connector_fixture(),
            direction: DataplaneInitCommandDirection::Push {
                data_address: Some(empty_address()),
            },
        })
        .await;
        assert_eq!(get_strategy(context).handler_name(), "ProviderPush");
    }
}
