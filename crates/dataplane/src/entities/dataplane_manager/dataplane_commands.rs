/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::config_builder::DataplaneConfigBuilder;
use crate::entities::dataplane_manager::driver_factory::DataplaneDriver;
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, EditDataplaneTransferDto, InteractionMode, TransferState,
};
use crate::{DataplaneManager, DataplaneResponse};
use connector::ConnectorInstanceDto;
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug)]
pub(super) struct CommandContext<'a> {
    pub(super) process_id: Urn,
    state: &'a TransferState,
    mode: &'a InteractionMode,
    driver: &'a DataplaneDriver,
    connector: Option<&'a ConnectorInstanceDto>,
}

impl<'a> CommandContext<'a> {
    pub(super) fn from_connector(
        process: &'a DataplaneTransferDto,
        driver: &'a DataplaneDriver,
        connector: Option<&'a ConnectorInstanceDto>,
    ) -> Outcome<Self> {
        Ok(Self {
            process_id: process.inner.id.parse()?,
            state: &process.inner.state,
            mode: &process.inner.interaction_mode,
            driver,
            connector,
        })
    }

    fn is_state(&self, expected: TransferState) -> bool {
        *self.state == expected
    }

    fn is_push(&self) -> bool {
        *self.mode == InteractionMode::Push
    }
}

impl DataplaneManager {
    // ─── Individual command handlers ───

    /// INIT → CONFIGURING: freeze initial ingress/egress config from connector
    pub(super) async fn cmd_init(
        &self,
        process: &DataplaneTransferDto,
        ctx: &CommandContext<'_>,
    ) -> Outcome<DataplaneResponse> {
        if ctx.is_state(TransferState::Init) {
            // Config was already computed in handle_creation, just persist and transition
            let builder = DataplaneConfigBuilder::from_process(process);
            self.update_state_and_config(
                &ctx.process_id,
                TransferState::Configuring,
                Some(builder.ingress),
                Some(builder.egress),
            )
            .await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// CONFIGURING → AUTH: driver authenticates
    pub(super) async fn cmd_auth(&self, ctx: &CommandContext<'_>) -> Outcome<DataplaneResponse> {
        if ctx.is_state(TransferState::Configuring) {
            ctx.driver.auth_driver.perform_auth(ctx.connector).await?;
            self.update_state(&ctx.process_id, TransferState::Auth)
                .await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// AUTH → READY
    pub(super) async fn cmd_ready(&self, ctx: &CommandContext<'_>) -> Outcome<DataplaneResponse> {
        if ctx.is_state(TransferState::Auth) {
            self.update_state(&ctx.process_id, TransferState::Ready)
                .await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// READY → SUBSCRIBING (PUSH only): driver subscribes to upstream
    pub(super) async fn cmd_subscribing(
        &self,
        ctx: &CommandContext<'_>,
    ) -> Outcome<DataplaneResponse> {
        if ctx.is_state(TransferState::Ready) && ctx.is_push() {
            self.subscribe_or_terminate(ctx).await?;
            self.update_state(&ctx.process_id, TransferState::Subscribing)
                .await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// READY or STOPPED → STARTED (PULL: direct) or via SUBSCRIBING (PUSH: autonomous)
    pub(super) async fn cmd_started(&self, ctx: &CommandContext<'_>) -> Outcome<DataplaneResponse> {
        match (ctx.mode, ctx.state) {
            // PULL: Ready → Started directly
            (InteractionMode::Pull, &TransferState::Ready) => {
                self.update_state(&ctx.process_id, TransferState::Started)
                    .await?;
            }
            // PULL: Stopped → Started (resume after suspension)
            (InteractionMode::Pull, &TransferState::Stopped) => {
                self.update_state(&ctx.process_id, TransferState::Started)
                    .await?;
            }
            // PUSH: Ready → subscribe → Subscribing (autonomous will chain to Started)
            (InteractionMode::Push, &TransferState::Ready) => {
                self.subscribe_or_terminate(ctx).await?;
                self.update_state(&ctx.process_id, TransferState::Subscribing)
                    .await?;
            }
            // PUSH: Stopped → re-subscribe → Subscribing (resume after suspension)
            (InteractionMode::Push, &TransferState::Stopped) => {
                self.subscribe_or_terminate(ctx).await?;
                self.update_state(&ctx.process_id, TransferState::Subscribing)
                    .await?;
            }
            // PUSH: Subscribing → Started (autonomous transition target)
            (InteractionMode::Push, &TransferState::Subscribing) => {
                self.update_state(&ctx.process_id, TransferState::Started)
                    .await?;
            }
            _ => {}
        }
        Ok(DataplaneResponse::Ok)
    }

    /// STARTED → UNSUBSCRIBING (PUSH only): driver unsubscribes
    pub(super) async fn cmd_unsubscribing(
        &self,
        ctx: &CommandContext<'_>,
    ) -> Outcome<DataplaneResponse> {
        if ctx.is_state(TransferState::Started) && ctx.is_push() {
            self.unsubscribe_or_terminate(ctx).await?;
            self.update_state(&ctx.process_id, TransferState::Unsubscribing)
                .await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// STARTED → STOPPED (PULL: direct) or STARTED → UNSUBSCRIBING → STOPPED (PUSH: via autonomous)
    pub(super) async fn cmd_stopped(&self, ctx: &CommandContext<'_>) -> Outcome<DataplaneResponse> {
        match (ctx.mode, ctx.state) {
            // PULL: Started → Stopped directly
            (InteractionMode::Pull, &TransferState::Started) => {
                self.update_state(&ctx.process_id, TransferState::Stopped)
                    .await?;
            }
            // PUSH: Started → unsubscribe → Unsubscribing (autonomous will chain to Stopped)
            (InteractionMode::Push, &TransferState::Started) => {
                self.unsubscribe_or_terminate(ctx).await?;
                self.update_state(&ctx.process_id, TransferState::Unsubscribing)
                    .await?;
            }
            // PUSH: Unsubscribing → Stopped (autonomous transition target)
            (InteractionMode::Push, &TransferState::Unsubscribing) => {
                self.update_state(&ctx.process_id, TransferState::Stopped)
                    .await?;
            }
            _ => {}
        }
        Ok(DataplaneResponse::Ok)
    }

    /// Calls `perform_subscribe`; stores response in flow_control, or persists TERMINATED on failure.
    async fn subscribe_or_terminate(&self, ctx: &CommandContext<'_>) -> Outcome<()> {
        match ctx
            .driver
            .lifecycle_driver
            .perform_subscribe(ctx.connector)
            .await
        {
            Ok(response) => {
                if !response.is_null() {
                    self.dataplane_entity
                        .put_dataplane_transfer_by_id(
                            &ctx.process_id,
                            &EditDataplaneTransferDto {
                                flow_control: Some(response),
                                ..Default::default()
                            },
                        )
                        .await?;
                }
                Ok(())
            }
            Err(e) => {
                self.update_state(&ctx.process_id, TransferState::Terminated)
                    .await?;
                Err(e)
            }
        }
    }

    /// Calls `perform_unsubscribe`; on failure persists TERMINATED state before returning the error.
    async fn unsubscribe_or_terminate(&self, ctx: &CommandContext<'_>) -> Outcome<()> {
        if let Err(e) = ctx
            .driver
            .lifecycle_driver
            .perform_unsubscribe(ctx.connector)
            .await
        {
            self.update_state(&ctx.process_id, TransferState::Terminated)
                .await?;
            return Err(e);
        }
        Ok(())
    }

    /// ANY → TERMINATED: cleanup (emergency unsubscribe if PUSH active)
    pub(super) async fn cmd_terminated(
        &self,
        ctx: &CommandContext<'_>,
    ) -> Outcome<DataplaneResponse> {
        if ctx.is_push() {
            match ctx.state {
                TransferState::Started | TransferState::Subscribing => {
                    let _ = ctx
                        .driver
                        .lifecycle_driver
                        .perform_unsubscribe(ctx.connector)
                        .await;
                }
                _ => {}
            }
        }
        self.update_state(&ctx.process_id, TransferState::Terminated)
            .await?;
        Ok(DataplaneResponse::Ok)
    }
}
