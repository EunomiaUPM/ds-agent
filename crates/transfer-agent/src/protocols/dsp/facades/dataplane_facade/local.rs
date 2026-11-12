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

use crate::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;

use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use dataplane::DataplaneManager;
use std::sync::Arc;
use ymir::errors::Outcome;

/// Drives the dataplane manager in-process; the dataplane always runs inside transfer.
pub struct DataPlaneLocalFacade {
    dataplane_manager: Arc<DataplaneManager>,
}

impl DataPlaneLocalFacade {
    pub fn new(dataplane_manager: Arc<DataplaneManager>) -> DataPlaneLocalFacade {
        DataPlaneLocalFacade { dataplane_manager }
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DataPlaneLocalFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_request_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.request_strategy()
            .on_request_pre(ctx, &self.dataplane_manager)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_request_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_request_post(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_start_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_start_pre(ctx, &self.dataplane_manager)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_start_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_start_post(ctx, &self.dataplane_manager)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_suspension_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_suspend_pre(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_suspension_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_suspend_post(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_completion_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_complete_pre(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_completion_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_complete_post(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_termination_pre(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_terminate_pre(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "dataplane"))]
    async fn on_transfer_termination_post(
        &self,
        ctx: &DataplaneHookContext,
    ) -> Outcome<Option<DataAddressDto>> {
        ctx.strategy()
            .on_terminate_post(ctx, &self.dataplane_manager)
            .await?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::protocol::{TransferDirection, TransferRole};
    use crate::protocols::dsp::entities::context_common::TransferContextConnectorRole;
    use common::test_utils::config_fixtures::transfer_config_fixture;
    use connector::MockConnectorInstanceFacadeTrait;
    use dataplane::data::sea_orm::orm::dataplane_transfers;
    use dataplane::entities::dataplane_transfers::{
        DataplaneTransferDto, InteractionMode, TransferRole as DpRole, TransferState,
    };
    use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
    use serde_json::json;
    use std::str::FromStr;
    use std::sync::Mutex;
    use urn::Urn;

    fn make_facade_with_state(
        role: DpRole,
        mode: InteractionMode,
        initial: TransferState,
    ) -> (DataPlaneLocalFacade, Arc<Mutex<Vec<TransferState>>>) {
        let history = Arc::new(Mutex::new(vec![initial]));
        let mut svc = MockDataplaneTransferServiceTrait::new();
        let hist_get = history.clone();
        let role_clone = role.clone();
        let mode_clone = mode.clone();
        svc.expect_get_by_process_id()
            .returning(move |_, tp_urn| {
                let current = hist_get.lock().unwrap().last().cloned().unwrap();
                Ok(DataplaneTransferDto {
                    inner: dataplane_transfers::Model {
                        user_id: "tenant-1".to_string(),
                        user_role: ymir::types::oauth::RolePath::root(),
                        visibility: ymir::types::participants::Visibility::Private,
                        id: "urn:dataplane-transfer:hook-test-1".to_string(),
                        transfer_process_id: tp_urn.to_string(),
                        role: role_clone.clone(),
                        interaction_mode: mode_clone.clone(),
                        state: current,
                        connector_instance_id: None,
                        ingress_config: json!("NoOp"),
                        egress_config: json!("NoOp"),
                        flow_control: None,
                        created_at: chrono::Utc::now().into(),
                        updated_at: None,
                    },
                    fields: Default::default(),
                    logs: vec![],
                })
            });
        let hist_edit = history.clone();
        let role_edit = role.clone();
        let mode_edit = mode.clone();
        svc.expect_edit().returning(move |_, tp_urn, edit| {
            let mut guard = hist_edit.lock().unwrap();
            if let Some(new_st) = &edit.state {
                guard.push(new_st.clone());
            }
            let current = guard.last().cloned().unwrap();
            Ok(DataplaneTransferDto {
                inner: dataplane_transfers::Model {
                    user_id: "tenant-1".to_string(),
                    user_role: ymir::types::oauth::RolePath::root(),
                    visibility: ymir::types::participants::Visibility::Private,
                    id: "urn:dataplane-transfer:hook-test-1".to_string(),
                    transfer_process_id: tp_urn.to_string(),
                    role: role_edit.clone(),
                    interaction_mode: mode_edit.clone(),
                    state: current,
                    connector_instance_id: None,
                    ingress_config: json!("NoOp"),
                    egress_config: json!("NoOp"),
                    flow_control: None,
                    created_at: chrono::Utc::now().into(),
                    updated_at: None,
                },
                fields: Default::default(),
                logs: vec![],
            })
        });

        let mut manager = DataplaneManager::new(
            Arc::new(svc),
            Arc::new(MockConnectorInstanceFacadeTrait::new()),
            transfer_config_fixture(),
        );
        if matches!(role, DpRole::Provider) {
            use dataplane::engine::dataplane_drivers::authentication::no_op::NoOpAuthenticator;
            use dataplane::engine::dataplane_drivers::configuration::no_op::NoOpProxyConfigurator;
            use dataplane::engine::dataplane_drivers::pubsub::no_op::NoOpPubSubscriber;
            use dataplane::engine::dataplane_drivers::DataplaneDriver;
            use dataplane::engine::dataplane_manager::dataplane_driver_factory::MockDataplaneDriverFactoryTrait;

            let mut mock_factory = MockDataplaneDriverFactoryTrait::new();
            let mode_for_driver = mode.clone();
            mock_factory.expect_get_or_create_driver().returning(move |_| {
                Ok(DataplaneDriver {
                    authenticator: Arc::new(NoOpAuthenticator),
                    proxy_configurator: Arc::new(NoOpProxyConfigurator),
                    subscriber: match mode_for_driver {
                        InteractionMode::Pull => None,
                        InteractionMode::Push => Some(Arc::new(NoOpPubSubscriber)),
                    },
                })
            });
            manager = manager.with_driver_factory(Arc::new(mock_factory));
        }
        (DataPlaneLocalFacade::new(Arc::new(manager)), history)
    }

    #[tokio::test]
    async fn completion_and_termination_drive_provider_pull_to_terminated() {
        let (facade, history) =
            make_facade_with_state(DpRole::Provider, InteractionMode::Pull, TransferState::Started);
        let ctx = DataplaneHookContext {
            process_id: Urn::from_str("urn:transfer-process:hook-1").unwrap(),
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            role: TransferRole::Provider,
            transfer_direction: TransferDirection::Pull,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            data_address: None,
            resolved_data_address: None,
            is_restart: false,
        };

        facade.on_transfer_completion_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Started,
                TransferState::Stopped,
                TransferState::Terminated
            ]
        );

        *history.lock().unwrap() = vec![TransferState::Started];
        facade.on_transfer_termination_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Started,
                TransferState::Stopped,
                TransferState::Terminated
            ]
        );
    }

    #[tokio::test]
    async fn consumer_push_lifecycle_uses_subscribing_unsubscribing_and_terminating() {
        let (facade, history) =
            make_facade_with_state(DpRole::Consumer, InteractionMode::Push, TransferState::Ready);
        let mut ctx = DataplaneHookContext {
            process_id: Urn::from_str("urn:transfer-process:hook-2").unwrap(),
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            role: TransferRole::Consumer,
            transfer_direction: TransferDirection::Push,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            data_address: None,
            resolved_data_address: None,
            is_restart: false,
        };

        // Start: Ready -> Subscribing -> Started (via real DataplaneDriverFactory + NoOpPubSubscriber)
        facade.on_transfer_start_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Ready,
                TransferState::Subscribing,
                TransferState::Started
            ]
        );

        // Suspend: Started -> Unsubscribing -> Stopped
        facade.on_transfer_suspension_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Ready,
                TransferState::Subscribing,
                TransferState::Started,
                TransferState::Unsubscribing,
                TransferState::Stopped
            ]
        );

        // Resume: Stopped -> Subscribing -> Started
        ctx.is_restart = true;
        facade.on_transfer_start_post(&ctx).await.unwrap();
        assert_eq!(
            history.lock().unwrap().last().cloned(),
            Some(TransferState::Started)
        );

        // Terminate from Started: passes through Unsubscribing -> Stopped -> Terminated
        ctx.is_restart = false;
        *history.lock().unwrap() = vec![TransferState::Started];
        facade.on_transfer_termination_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Started,
                TransferState::Unsubscribing,
                TransferState::Stopped,
                TransferState::Terminated
            ]
        );
    }

    #[tokio::test]
    async fn provider_push_lifecycle_symmetric_with_consumer_push() {
        let (facade, history) =
            make_facade_with_state(DpRole::Provider, InteractionMode::Push, TransferState::Ready);
        let ctx = DataplaneHookContext {
            process_id: Urn::from_str("urn:transfer-process:hook-3").unwrap(),
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            role: TransferRole::Provider,
            transfer_direction: TransferDirection::Push,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            data_address: None,
            resolved_data_address: None,
            is_restart: false,
        };

        facade.on_transfer_start_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Ready,
                TransferState::Subscribing,
                TransferState::Started
            ]
        );

        facade.on_transfer_completion_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Ready,
                TransferState::Subscribing,
                TransferState::Started,
                TransferState::Unsubscribing,
                TransferState::Stopped,
                TransferState::Terminated
            ]
        );
    }

    #[tokio::test]
    async fn consumer_pull_lifecycle_configures_on_first_start_and_restarts_from_stopped() {
        let (facade, history) =
            make_facade_with_state(DpRole::Consumer, InteractionMode::Pull, TransferState::Init);
        let mut ctx = DataplaneHookContext {
            process_id: Urn::from_str("urn:transfer-process:hook-4").unwrap(),
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            role: TransferRole::Consumer,
            transfer_direction: TransferDirection::Pull,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            data_address: Some(
                DataAddressDto {
                    endpoint_type: "https://w3id.org/idsa/v4.1/HTTP".to_string(),
                    endpoint: Some("http://provider.example/dataplane/proxy/p1".to_string()),
                    endpoint_properties: None,
                }
                .into(),
            ),
            resolved_data_address: None,
            is_restart: false,
        };

        // First start: Init -> Configuring -> Auth -> Ready -> Started
        facade.on_transfer_start_post(&ctx).await.unwrap();
        assert_eq!(
            *history.lock().unwrap(),
            vec![
                TransferState::Init,
                TransferState::Configuring,
                TransferState::Auth,
                TransferState::Ready,
                TransferState::Started
            ]
        );

        // Suspend: Started -> Stopped
        facade.on_transfer_suspension_post(&ctx).await.unwrap();
        assert_eq!(
            history.lock().unwrap().last().cloned(),
            Some(TransferState::Stopped)
        );

        // Restart: Stopped -> Started (SetStarted accepted because state == Stopped)
        ctx.is_restart = true;
        facade.on_transfer_start_post(&ctx).await.unwrap();
        assert_eq!(
            history.lock().unwrap().last().cloned(),
            Some(TransferState::Started)
        );

        // Complete: Started -> Stopped -> Terminated
        ctx.is_restart = false;
        facade.on_transfer_completion_post(&ctx).await.unwrap();
        assert_eq!(
            history.lock().unwrap().last().cloned(),
            Some(TransferState::Terminated)
        );
    }
}
