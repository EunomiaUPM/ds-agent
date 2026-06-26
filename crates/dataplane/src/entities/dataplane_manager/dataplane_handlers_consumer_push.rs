use crate::entities::dataplane_manager::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneHandlerConsumerPush {
    dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneHandlerConsumerPush {
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
}

#[async_trait::async_trait]
impl DataplaneCommandStateMachine for DataplaneHandlerConsumerPush {
    fn handler_name(&self) -> &'static str {
        "ConsumerPush"
    }
    fn dataplane_entity(&self) -> Arc<dyn DataplaneTransfersEntitiesTrait> {
        self.dataplane_entity.clone()
    }
    fn connector_entity(&self) -> Arc<dyn ConnectorInstanceTrait> {
        self.connector_entity.clone()
    }
    fn transfer_config(&self) -> Arc<TransferConfig> {
        self.config.clone()
    }
    fn secret_store(&self) -> Option<Arc<dyn SecretStore>> {
        self.secret_store.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::DataplaneHandlerConsumerPush;
    use crate::data::entities::dataplane_transfers;
    use crate::entities::dataplane_drivers::authentication::no_op::NoOpAuthenticator;
    use crate::entities::dataplane_drivers::configuration::no_op::NoOpProxyConfigurator;
    use crate::entities::dataplane_drivers::pubsub::no_op::NoOpPubSubscriber;
    use crate::entities::dataplane_drivers::DataplaneDriver;
    use crate::entities::dataplane_manager::dataplane_commands::{
        DataplaneCommandStateMachine, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
    use crate::entities::dataplane_transfers::{
        DataplaneTransferDto, InteractionMode, MockDataplaneTransfersEntitiesTrait, TransferRole,
        TransferState,
    };
    use crate::DataplaneAddress;
    use common::test_utils::config_fixtures::transfer_config_fixture;
    use connector::{ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto};
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

    const DP_URN: &str = "urn:dataplane-transfer:1";
    const TP_URN: &str = "urn:transfer-process:1";

    // The address where this consumer will receive pushed data.
    fn push_endpoint_fixture() -> DataplaneAddress {
        DataplaneAddress {
            endpoint_type: "HTTP".to_string(),
            endpoint: "http://consumer-endpoint.com/receive".to_string(),
            authorization_type: Some("Bearer".to_string()),
            authorization: Some("consumer-token".to_string()),
        }
    }

    fn dto(state: TransferState) -> DataplaneTransferDto {
        DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                id: DP_URN.to_string(),
                transfer_process_id: TP_URN.to_string(),
                role: TransferRole::Consumer,
                interaction_mode: InteractionMode::Push,
                state,
                connector_instance_id: None,
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

    fn handler(
        entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait>,
    ) -> DataplaneHandlerConsumerPush {
        DataplaneHandlerConsumerPush::new(
            entity,
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            None,
        )
    }

    async fn init_context(
        entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait>,
    ) -> DataplaneContext {
        DataplaneContext::from_init(
            entity,
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            DataplaneInitCommandTypes::AsConsumer {
                transfer_process_id: Urn::from_str(TP_URN).unwrap(),
                direction: DataplaneInitCommandDirection::Push {
                    data_address: Some(push_endpoint_fixture()),
                },
            },
        )
        .await
        .unwrap()
    }

    fn driver_with_subscriber() -> DataplaneDriver {
        DataplaneDriver {
            authenticator: Arc::new(NoOpAuthenticator),
            proxy_configurator: Arc::new(NoOpProxyConfigurator),
            subscriber: Some(Arc::new(NoOpPubSubscriber)),
        }
    }

    fn expect_create(mock: &mut MockDataplaneTransfersEntitiesTrait) {
        mock.expect_create_dataplane_transfer()
            .times(1)
            .returning(|_| Ok(dto(TransferState::Init)));
    }

    fn expect_put(mock: &mut MockDataplaneTransfersEntitiesTrait, expected: TransferState) {
        let check = expected.clone();
        mock.expect_put_dataplane_transfer_by_id()
            .times(1)
            .withf(move |_, edit| edit.state == Some(check.clone()))
            .returning(move |_, _| Ok(dto(expected.clone())));
    }

    // set_configuring ───────────────────────────────────────────────────────

    // set_configuring is atomic: configure proxy (NoOp) - put(Configuring).
    // Does NOT proceed to auth or ready.
    #[tokio::test]
    async fn test_set_configuring_persists_configuring_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_configuring(context).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(
            ctx.dataplane_process().inner.state,
            TransferState::Configuring
        );
        assert!(ctx.driver().is_some());
        assert!(ctx.connector_instance().is_none());
        let addr = ctx
            .forward_dataplane_address()
            .expect("push endpoint must be preserved");
        assert_eq!(addr.endpoint, "http://consumer-endpoint.com/receive");
    }

    // set_auth ──────────────────────────────────────────────────────────────

    // set_auth is atomic: NoOp authentication - put(Auth). Does NOT proceed to ready.
    #[tokio::test]
    async fn test_set_auth_persists_auth_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Auth);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_auth(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Auth
        );
    }

    // set_ready ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_ready_persists_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Ready);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_ready(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Ready
        );
    }

    // set_started ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_started_persists_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Started);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_started(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Started
        );
    }

    // set_stopped ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_stopped_persists_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Stopped);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_stopped(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Stopped
        );
    }

    // set_terminating ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_terminating_persists_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Terminated);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_terminating(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Terminated
        );
    }

    // set_subscribing ───────────────────────────────────────────────────────

    // With a driver carrying a subscriber: put(Subscribing) - subscribe - set_started - put(Started).
    #[tokio::test]
    async fn test_set_subscribing_with_driver_activates_push() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Subscribing);
        expect_put(&mut mock, TransferState::Started);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let mut context = init_context(entity.clone()).await;
        context.set_driver(driver_with_subscriber());

        let result = handler(entity).set_subscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Started
        );
    }

    // Without a driver (init context default): set_subscribing is a no-op.
    #[tokio::test]
    async fn test_set_subscribing_without_driver_is_noop() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        // no expect_put — unexpected calls panic

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_subscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Init
        );
    }

    // set_unsubscribing ─────────────────────────────────────────────────────

    // With a driver carrying a subscriber: put(Unsubscribing) - unsubscribe - set_stopped - put(Stopped).
    #[tokio::test]
    async fn test_set_unsubscribing_with_driver_deactivates_push() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Unsubscribing);
        expect_put(&mut mock, TransferState::Stopped);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let mut context = init_context(entity.clone()).await;
        context.set_driver(driver_with_subscriber());

        let result = handler(entity).set_unsubscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Stopped
        );
    }

    // Without a driver: set_unsubscribing is a no-op.
    #[tokio::test]
    async fn test_set_unsubscribing_without_driver_is_noop() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_unsubscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Init
        );
    }
}
