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

use crate::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceServiceTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneHandlerConsumerPush {
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    connector_service: Arc<dyn ConnectorInstanceServiceTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneHandlerConsumerPush {
    pub fn new(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        connector_service: Arc<dyn ConnectorInstanceServiceTrait>,
        config: Arc<TransferConfig>,
        secret_store: Option<Arc<dyn SecretStore>>,
    ) -> Self {
        Self {
            dataplane_service,
            connector_service,
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
    fn dataplane_service(&self) -> Arc<dyn DataplaneTransferServiceTrait> {
        self.dataplane_service.clone()
    }
    fn connector_service(&self) -> Arc<dyn ConnectorInstanceServiceTrait> {
        self.connector_service.clone()
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
    use crate::data::sea_orm::orm::dataplane_transfers;
    use crate::engine::dataplane_drivers::authentication::no_op::NoOpAuthenticator;
    use crate::engine::dataplane_drivers::configuration::no_op::NoOpProxyConfigurator;
    use crate::engine::dataplane_drivers::pubsub::no_op::NoOpPubSubscriber;
    use crate::engine::dataplane_drivers::DataplaneDriver;
    use crate::engine::dataplane_manager::dataplane_commands::{
        DataplaneCommandStateMachine, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
    use crate::entities::dataplane_transfers::{
        DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
    };
    use crate::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
    use crate::DataplaneAddress;
    use common::test_utils::config_fixtures::transfer_config_fixture;
    use connector::{
        ConnectorInstanceDto, ConnectorInstanceServiceTrait, ConnectorInstantiationDto,
    };
    use mockall::mock;
    use std::str::FromStr;
    use std::sync::Arc;
    use urn::Urn;
    use ymir::errors::Outcome;

    mock! {
        pub ConnectorMock {}
        #[async_trait::async_trait]
        impl ConnectorInstanceServiceTrait for ConnectorMock {
            async fn get_instance_by_id(&self, scope: &common::auth::AccessScope, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn get_instance_by_distribution(&self, scope: &common::auth::AccessScope, distribution_id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn upsert_instance(&self, scope: &common::auth::AccessScope, dto: &mut ConnectorInstantiationDto) -> Outcome<ConnectorInstanceDto>;
            async fn delete_instance_by_id(&self, scope: &common::auth::AccessScope, id: &Urn) -> Outcome<()>;
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
                tenant_id: "tenant-1".to_string(),
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
        entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait>,
    ) -> DataplaneHandlerConsumerPush {
        DataplaneHandlerConsumerPush::new(
            entity,
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            None,
        )
    }

    async fn init_context(
        entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait>,
    ) -> DataplaneContext {
        DataplaneContext::from_init(
            entity,
            Arc::new(MockConnectorMock::new()),
            transfer_config_fixture(),
            DataplaneInitCommandTypes::AsConsumer {
                tenant_id: "tenant-1".to_string(),
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

    fn expect_create(mock: &mut MockDataplaneTransferServiceTrait) {
        mock.expect_create()
            .times(1)
            .returning(|_, _| Ok(dto(TransferState::Init)));
    }

    fn expect_put(mock: &mut MockDataplaneTransferServiceTrait, expected: TransferState) {
        let check = expected.clone();
        mock.expect_edit()
            .times(1)
            .withf(move |_, _, edit| edit.state == Some(check.clone()))
            .returning(move |_, _, _| Ok(dto(expected.clone())));
    }

    // set_configuring ───────────────────────────────────────────────────────

    // set_configuring is atomic: configure proxy (NoOp) - put(Configuring).
    // Does NOT proceed to auth or ready.
    #[tokio::test]
    async fn test_set_configuring_persists_configuring_state() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Auth);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Ready);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Started);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Stopped);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Terminated);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_terminating(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Terminated
        );
    }

    // set_subscribing ───────────────────────────────────────────────────────

    // With a driver carrying a subscriber: put(Subscribing) - subscribe - set_started -
    // put(Started).
    #[tokio::test]
    async fn test_set_subscribing_with_driver_activates_push() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Subscribing);
        expect_put(&mut mock, TransferState::Started);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        // no expect_put — unexpected calls panic

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_subscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Init
        );
    }

    // set_unsubscribing ─────────────────────────────────────────────────────

    // With a driver carrying a subscriber: put(Unsubscribing) - unsubscribe - set_stopped -
    // put(Stopped).
    #[tokio::test]
    async fn test_set_unsubscribing_with_driver_deactivates_push() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Unsubscribing);
        expect_put(&mut mock, TransferState::Stopped);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_unsubscribing(context).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Init
        );
    }
}
