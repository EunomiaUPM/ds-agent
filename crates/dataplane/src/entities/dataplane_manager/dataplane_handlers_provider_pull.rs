use crate::entities::dataplane_manager::dataplane_commands::{
    set_configuring_helper, DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactory;
use crate::entities::dataplane_manager::dataplane_proxy::DataplaneProxy;
use crate::entities::dataplane_transfers::{EditDataplaneTransferDto, TransferState};
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct DataplaneHandlerProviderPull {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}

impl DataplaneHandlerProviderPull {
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
}

#[async_trait::async_trait]
impl DataplaneCommandStateMachine for DataplaneHandlerProviderPull {
    fn handler_name(&self) -> &'static str {
        "ProviderPull"
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
    async fn set_init(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = set_configuring_helper(self.dataplane_entity(), context).await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        Ok(ctx)
    }
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = set_configuring_helper(self.dataplane_entity(), context).await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        let ctx = self.set_started(ctx).await?;
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::DataplaneHandlerProviderPull;
    use crate::data::entities::dataplane_transfers;
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
    use connector::{
        AuthenticationConfig, ConnectorInstanceDto, ConnectorInstanceTrait,
        ConnectorInstantiationDto, ConnectorMetadata, HttpSpec, InteractionConfig, ProtocolSpec,
        PullLifecycle, TemplateVecString,
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

    const DP_URN: &str = "urn:dataplane-transfer:1";
    const TP_URN: &str = "urn:transfer-process:1";
    const CONNECTOR_URN: &str = "urn:connector-instance:1";

    // A provider-pull connector: NoAuth + HTTP data access.
    fn connector_fixture() -> ConnectorInstanceDto {
        ConnectorInstanceDto {
            id: Urn::from_str(CONNECTOR_URN).unwrap(),
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
                    url_template: "http://data-source.internal/api".to_string(),
                    method: TemplateVecString::Value(vec!["GET".to_string()]),
                    headers: None,
                    body_template: None,
                }),
            }),
            distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
        }
    }

    // The proxy address the consumer will use to pull data from this provider.
    fn proxy_address_fixture() -> DataplaneAddress {
        DataplaneAddress {
            endpoint_type: "HTTP".to_string(),
            endpoint: "http://dataplane.example.com/proxy/transfer".to_string(),
            authorization_type: Some("Bearer".to_string()),
            authorization: Some("proxy-token-xyz".to_string()),
        }
    }

    fn dto(state: TransferState) -> DataplaneTransferDto {
        DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                id: DP_URN.to_string(),
                transfer_process_id: TP_URN.to_string(),
                role: TransferRole::Provider,
                interaction_mode: InteractionMode::Pull,
                state,
                connector_instance_id: Some(CONNECTOR_URN.to_string()),
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
    ) -> DataplaneHandlerProviderPull {
        DataplaneHandlerProviderPull::new(
            entity,
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
        )
    }

    async fn init_context(
        entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait>,
    ) -> DataplaneContext {
        DataplaneContext::from_init(
            entity,
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            DataplaneInitCommandTypes::AsProvider {
                transfer_process_id: Urn::from_str(TP_URN).unwrap(),
                connector_instance: connector_fixture(),
                direction: DataplaneInitCommandDirection::Pull {
                    data_address: proxy_address_fixture(),
                },
            },
        )
        .await
        .unwrap()
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

    // ── set_configuring ───────────────────────────────────────────────────────

    // set_configuring is atomic: configure proxy (NoOp) - put(Configuring).
    // Does NOT proceed to auth or ready.
    #[tokio::test]
    async fn test_set_configuring_persists_configuring_state_and_preserves_connector() {
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
        // connector must survive configuring — it describes the provider's data source
        let conn = ctx
            .connector_instance()
            .expect("connector instance must be preserved");
        assert_eq!(conn.id, Urn::from_str(CONNECTOR_URN).unwrap());
        let addr = ctx
            .forward_dataplane_address()
            .expect("proxy address must be preserved");
        assert_eq!(addr.endpoint, "http://dataplane.example.com/proxy/transfer");
    }

    // ── set_auth ──────────────────────────────────────────────────────────────

    // set_auth is atomic: NoAuth - NoOp authentication - put(Auth). Does NOT proceed to ready.
    #[tokio::test]
    async fn test_set_auth_persists_auth_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Auth);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_auth(context).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.dataplane_process().inner.state, TransferState::Auth);
        assert!(ctx.connector_instance().is_some());
    }

    // ── set_ready ─────────────────────────────────────────────────────────────

    // set_ready must call put exactly once with state=Ready and return the updated context.
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

    // ── set_started ───────────────────────────────────────────────────────────

    // set_started must call put exactly once with state=Started and return the updated context.
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

    // ── set_stopped ───────────────────────────────────────────────────────────

    // set_stopped must call put exactly once with state=Stopped and return the updated context.
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

    // ── set_terminating ───────────────────────────────────────────────────────

    // set_terminating must call put exactly once with state=Terminated and return the updated context.
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

    // ── set_subscribing / set_unsubscribing ───────────────────────────────────

    // Pull mode has no subscriber: set_subscribing must be a no-op — no put is called
    // and the context is returned unchanged. (driver is None when built from from_init)
    #[tokio::test]
    async fn test_set_subscribing_is_noop_for_pull() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        // no expect_put — any unexpected call makes the mock panic

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_subscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Init
        );
    }

    // Pull mode has no subscriber: set_unsubscribing must also be a no-op.
    #[tokio::test]
    async fn test_set_unsubscribing_is_noop_for_pull() {
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
