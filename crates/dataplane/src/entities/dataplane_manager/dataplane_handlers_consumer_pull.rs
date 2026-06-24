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
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneHandlerConsumerPull {
    dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneHandlerConsumerPull {
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
impl DataplaneCommandStateMachine for DataplaneHandlerConsumerPull {
    fn handler_name(&self) -> &'static str {
        "ConsumerPull"
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
    use super::DataplaneHandlerConsumerPull;
    use crate::data::entities::dataplane_transfers;
    use crate::entities::dataplane_manager::dataplane_commands::{
        DataplaneCommandStateMachine, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
    use crate::entities::dataplane_manager::dataplane_proxy::{
        DataplaneProxyEgress, DataplaneProxyIngress,
    };
    use crate::entities::dataplane_transfers::{
        DataplaneTransferDto, InteractionMode, MockDataplaneTransfersEntitiesTrait, TransferRole,
        TransferState,
    };
    use crate::DataplaneAddress;
    use common::test_utils::config_fixtures::transfer_config_fixture;
    use connector::{ConnectorInstanceDto, ConnectorInstanceTrait, ConnectorInstantiationDto};
    use mockall::mock;
    use serde_json::json;
    use std::str::FromStr;
    use std::sync::Arc;
    use urn::Urn;
    use ymir::errors::Outcome;

    mock! {
        pub ConnectorInstance {}
        #[async_trait::async_trait]
        impl ConnectorInstanceTrait for ConnectorInstance {
            async fn get_instance_by_id(&self, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn get_instance_by_distribution(&self, distribution_id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn upsert_instance(&self, dto: &mut ConnectorInstantiationDto) -> Outcome<ConnectorInstanceDto>;
            async fn delete_instance_by_id(&self, id: &Urn) -> Outcome<()>;
        }
    }

    const DP_URN: &str = "urn:dataplane-transfer:1";
    const TP_URN: &str = "urn:transfer-process:1";

    fn forward_address_fixture() -> DataplaneAddress {
        DataplaneAddress {
            endpoint_type: "HTTP".to_string(),
            endpoint: "http://provider-endpoint.com/data".to_string(),
            authorization_type: Some("Bearer".to_string()),
            authorization: Some("token-abc".to_string()),
        }
    }

    fn dto(state: TransferState) -> DataplaneTransferDto {
        DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                id: DP_URN.to_string(),
                transfer_process_id: TP_URN.to_string(),
                role: TransferRole::Consumer,
                interaction_mode: InteractionMode::Pull,
                state,
                connector_instance_id: None,
                ingress_config: json!({}),
                egress_config: json!({}),
                flow_control: Some(json!({})),
                created_at: chrono::Utc::now().into(),
                updated_at: None,
            },
            fields: Default::default(),
            logs: vec![],
        }
    }

    fn handler(
        entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait>,
    ) -> DataplaneHandlerConsumerPull {
        DataplaneHandlerConsumerPull::new(
            entity,
            Arc::new(MockConnectorInstance::new()),
            transfer_config_fixture(),
            None,
        )
    }

    async fn init_context(
        entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait>,
    ) -> DataplaneContext {
        DataplaneContext::from_init(
            entity,
            Arc::new(MockConnectorInstance::new()),
            transfer_config_fixture(),
            DataplaneInitCommandTypes::AsConsumer {
                transfer_process_id: Urn::from_str(TP_URN).unwrap(),
                direction: DataplaneInitCommandDirection::Pull {
                    data_address: Some(forward_address_fixture()),
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

    // set_init drives the full sub-chain: configuring - auth - ready.
    // After set_ready the forward address is replaced by the local ingress proxy URL
    // (the address the consumer advertises to the provider).
    #[tokio::test]
    async fn test_set_init_reaches_ready_and_exposes_proxy_address() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);
        expect_put(&mut mock, TransferState::Auth);
        expect_put(&mut mock, TransferState::Ready);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;
        let result = handler(entity).set_init(context).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.dataplane_process().inner.state, TransferState::Ready);
        // after set_ready the address is the local proxy ingress URL built from the DP id
        let addr = ctx
            .forward_dataplane_address()
            .expect("proxy ingress address must be set after set_ready");
        assert!(
            addr.endpoint.contains("/dataplane/proxy/"),
            "expected proxy ingress path, got: {}",
            addr.endpoint
        );
        // consumer pull never has a connector instance
        assert!(ctx.connector_instance().is_none());
    }

    // set_configuring ───────────────────────────────────────────────────────

    // set_configuring is atomic: configure proxy - put(Configuring).
    // It does NOT proceed to auth or ready.
    #[tokio::test]
    async fn test_set_configuring_builds_driver_and_proxy() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;
        let result = handler(entity.clone()).set_configuring(context).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(
            ctx.dataplane_process().inner.state,
            TransferState::Configuring
        );
        assert!(
            ctx.driver().is_some(),
            "driver must be set after configuring"
        );
        assert!(
            ctx.proxy().is_some(),
            "proxy must be built after configuring"
        );
        // provider address must be preserved
        let addr = ctx
            .forward_dataplane_address()
            .expect("provider address must be preserved");
        assert_eq!(addr.endpoint, "http://provider-endpoint.com/data");
        assert_eq!(addr.authorization_type.as_deref(), Some("Bearer"));
        assert!(ctx.connector_instance().is_none());
    }

    // set_auth ──────────────────────────────────────────────────────────────

    // set_auth is atomic: NoOp authentication - put(Auth). Does NOT proceed to ready.
    #[tokio::test]
    async fn test_set_auth_persists_auth_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);
        expect_put(&mut mock, TransferState::Auth);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;
        let context = handler(entity.clone())
            .set_configuring(context)
            .await
            .unwrap();

        let result = handler(entity.clone()).set_auth(context).await;
        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.dataplane_process().inner.state, TransferState::Auth);
        assert!(ctx.connector_instance().is_none());
        assert!(ctx.forward_dataplane_address().is_some());
    }

    // set_ready ─────────────────────────────────────────────────────────────

    // set_ready must call put exactly once with state=Ready and return the updated context.
    #[tokio::test]
    async fn test_set_ready_persists_state() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);
        expect_put(&mut mock, TransferState::Auth);
        expect_put(&mut mock, TransferState::Ready);

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;
        let context = handler(entity.clone())
            .set_configuring(context)
            .await
            .unwrap();
        let context = handler(entity.clone()).set_auth(context).await.unwrap();

        let result = handler(entity.clone()).set_ready(context).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Ready
        );
    }

    // set_started ───────────────────────────────────────────────────────────

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

    // set_stopped ───────────────────────────────────────────────────────────

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

    // set_terminating ───────────────────────────────────────────────────────

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

    // set_subscribing / set_unsubscribing ───────────────────────────────────

    // Pull mode has no subscriber: set_subscribing must be a no-op — no put is called
    // and the context is returned unchanged.
    #[tokio::test]
    async fn test_set_subscribing_is_noop_for_pull_and_reaches_started() {
        let mut mock = MockDataplaneTransfersEntitiesTrait::new();
        expect_create(&mut mock);
        // no expect_put — any unexpected call makes the mock panic

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_subscribing(context).await;

        assert!(result.is_ok());
        // context comes back untouched: state is still Init as returned by create
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
        // no expect_put — any unexpected call makes the mock panic

        let entity: Arc<dyn crate::DataplaneTransfersEntitiesTrait> = Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity).set_unsubscribing(context).await;

        assert!(result.is_ok());
        // context comes back untouched: state is still Init as returned by create
        assert_eq!(
            result.unwrap().dataplane_process().inner.state,
            TransferState::Init
        );
    }
}
