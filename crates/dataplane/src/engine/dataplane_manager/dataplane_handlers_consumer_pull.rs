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
    set_configuring_helper, DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceServiceTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneHandlerConsumerPull {
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    connector_service: Arc<dyn ConnectorInstanceServiceTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneHandlerConsumerPull {
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
impl DataplaneCommandStateMachine for DataplaneHandlerConsumerPull {
    fn handler_name(&self) -> &'static str {
        "ConsumerPull"
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

    async fn set_init(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context)
    }
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = set_configuring_helper(
            self.dataplane_service(),
            self.driver_factory().as_ref(),
            context,
        )
        .await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        let ctx = self.set_started(ctx).await?;
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::DataplaneHandlerConsumerPull;
    use crate::data::sea_orm::orm::dataplane_transfers;
    use crate::engine::dataplane_manager::dataplane_commands::{
        DataplaneCommandStateMachine, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
    };
    use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
    use crate::engine::dataplane_manager::dataplane_proxy::{
        DataplaneProxyEgress, DataplaneProxyIngress,
    };
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
    use serde_json::json;
    use std::str::FromStr;
    use std::sync::Arc;
    use urn::Urn;
    use ymir::errors::Outcome;

    mock! {
        pub ConnectorInstance {}
        #[async_trait::async_trait]
        impl ConnectorInstanceServiceTrait for ConnectorInstance {
            async fn get_instance_by_id(&self, scope: &common::auth::AccessScope, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn get_instance_by_distribution(&self, scope: &common::auth::AccessScope, distribution_id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
            async fn upsert_instance(&self, scope: &common::auth::AccessScope, dto: &mut ConnectorInstantiationDto) -> Outcome<ConnectorInstanceDto>;
            async fn delete_instance_by_id(&self, scope: &common::auth::AccessScope, id: &Urn) -> Outcome<()>;
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
                tenant_id: "tenant-1".to_string(),
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
        entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait>,
    ) -> DataplaneHandlerConsumerPull {
        DataplaneHandlerConsumerPull::new(
            entity,
            Arc::new(MockConnectorInstance::new()),
            transfer_config_fixture(),
            None,
        )
    }

    async fn init_context(
        entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait>,
    ) -> DataplaneContext {
        DataplaneContext::from_init(
            entity,
            Arc::new(MockConnectorInstance::new()),
            transfer_config_fixture(),
            DataplaneInitCommandTypes::AsConsumer {
                tenant_id: "tenant-1".to_string(),
                transfer_process_id: Urn::from_str(TP_URN).unwrap(),
                direction: DataplaneInitCommandDirection::Pull {
                    data_address: Some(forward_address_fixture()),
                },
            },
        )
        .await
        .unwrap()
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

    // ConsumerPull.set_init is a no-op: the full initialization flow is driven by
    // SetConfiguring (which the manager sends once the provider signals its address).
    #[tokio::test]
    async fn test_set_init_is_noop_for_consumer_pull() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        // no put expectations — set_init does not touch the DB

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;
        let result = handler(entity).set_init(context).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.dataplane_process().inner.state, TransferState::Init);
        assert!(ctx.connector_instance().is_none());
    }

    // set_configuring ───────────────────────────────────────────────────────

    // set_configuring drives the full consumer-pull flow atomically:
    // configure proxy → auth → ready → started.
    #[tokio::test]
    async fn test_set_configuring_builds_driver_and_proxy() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Configuring);
        expect_put(&mut mock, TransferState::Auth);
        expect_put(&mut mock, TransferState::Ready);
        expect_put(&mut mock, TransferState::Started);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;
        let result = handler(entity.clone()).set_configuring(context).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.dataplane_process().inner.state, TransferState::Started);
        assert!(
            ctx.driver().is_some(),
            "driver must be set after configuring"
        );
        assert!(
            ctx.proxy().is_some(),
            "proxy must be built after configuring"
        );
        assert!(ctx.connector_instance().is_none());
        // after set_ready/set_started the forward address is the local proxy ingress URL
        let addr = ctx
            .forward_dataplane_address()
            .expect("proxy ingress address must be set");
        assert!(
            addr.endpoint.contains("/dataplane/proxy/"),
            "expected proxy ingress path, got: {}",
            addr.endpoint
        );
    }

    // set_auth ──────────────────────────────────────────────────────────────

    // set_auth in isolation: no-op authentication → put(Auth).
    #[tokio::test]
    async fn test_set_auth_persists_auth_state() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Auth);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;

        let result = handler(entity.clone()).set_auth(context).await;
        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.dataplane_process().inner.state, TransferState::Auth);
        assert!(ctx.connector_instance().is_none());
    }

    // set_ready ─────────────────────────────────────────────────────────────

    // set_ready in isolation: put(Ready) exactly once.
    #[tokio::test]
    async fn test_set_ready_persists_state() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        expect_put(&mut mock, TransferState::Ready);

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
        let context = init_context(entity.clone()).await;

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

    // set_stopped must call put exactly once with state=Stopped and return the updated context.
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

    // set_terminating must call put exactly once with state=Terminated and return the updated
    // context.
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

    // set_subscribing / set_unsubscribing ───────────────────────────────────

    // Pull mode has no subscriber: set_subscribing must be a no-op — no put is called
    // and the context is returned unchanged.
    #[tokio::test]
    async fn test_set_subscribing_is_noop_for_pull_and_reaches_started() {
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        // no expect_put — any unexpected call makes the mock panic

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
        let mut mock = MockDataplaneTransferServiceTrait::new();
        expect_create(&mut mock);
        // no expect_put — any unexpected call makes the mock panic

        let entity: Arc<dyn crate::services::dataplane_transfers::DataplaneTransferServiceTrait> =
            Arc::new(mock);
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
