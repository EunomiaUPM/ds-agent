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
    ) -> (DataPlaneLocalFacade, Arc<Mutex<TransferState>>) {
        let state = Arc::new(Mutex::new(initial));
        let mut svc = MockDataplaneTransferServiceTrait::new();
        let st_get = state.clone();
        let role_clone = role.clone();
        let mode_clone = mode.clone();
        svc.expect_get_by_process_id()
            .returning(move |_, tp_urn| {
                Ok(DataplaneTransferDto {
                    inner: dataplane_transfers::Model {
                        user_id: "tenant-1".to_string(),
                        user_role: ymir::types::oauth::RolePath::root(),
                        visibility: ymir::types::participants::Visibility::Private,
                        id: "urn:dataplane-transfer:hook-test-1".to_string(),
                        transfer_process_id: tp_urn.to_string(),
                        role: role_clone.clone(),
                        interaction_mode: mode_clone.clone(),
                        state: st_get.lock().unwrap().clone(),
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
        let st_edit = state.clone();
        svc.expect_edit().returning(move |_, tp_urn, edit| {
            if let Some(new_st) = &edit.state {
                *st_edit.lock().unwrap() = new_st.clone();
            }
            Ok(DataplaneTransferDto {
                inner: dataplane_transfers::Model {
                    user_id: "tenant-1".to_string(),
                    user_role: ymir::types::oauth::RolePath::root(),
                    visibility: ymir::types::participants::Visibility::Private,
                    id: "urn:dataplane-transfer:hook-test-1".to_string(),
                    transfer_process_id: tp_urn.to_string(),
                    role: role.clone(),
                    interaction_mode: mode.clone(),
                    state: st_edit.lock().unwrap().clone(),
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

        let manager = Arc::new(DataplaneManager::new(
            Arc::new(svc),
            Arc::new(MockConnectorInstanceFacadeTrait::new()),
            transfer_config_fixture(),
        ));
        (DataPlaneLocalFacade::new(manager), state)
    }

    #[tokio::test]
    async fn completion_and_termination_drive_provider_pull_to_terminated() {
        let (facade, state) =
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
        assert_eq!(*state.lock().unwrap(), TransferState::Terminated);

        *state.lock().unwrap() = TransferState::Started;
        facade.on_transfer_termination_post(&ctx).await.unwrap();
        assert_eq!(*state.lock().unwrap(), TransferState::Terminated);
    }

    #[tokio::test]
    async fn consumer_push_lifecycle_transitions_ready_started_stopped_terminated() {
        let (facade, state) =
            make_facade_with_state(DpRole::Consumer, InteractionMode::Push, TransferState::Ready);
        let ctx = DataplaneHookContext {
            process_id: Urn::from_str("urn:transfer-process:hook-2").unwrap(),
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            role: TransferRole::Consumer,
            transfer_direction: TransferDirection::Push,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            data_address: None,
            resolved_data_address: None,
            is_restart: false,
        };

        facade.on_transfer_start_post(&ctx).await.unwrap();
        assert_eq!(*state.lock().unwrap(), TransferState::Started);

        facade.on_transfer_suspension_post(&ctx).await.unwrap();
        assert_eq!(*state.lock().unwrap(), TransferState::Stopped);

        facade.on_transfer_termination_post(&ctx).await.unwrap();
        assert_eq!(*state.lock().unwrap(), TransferState::Terminated);
    }
}
