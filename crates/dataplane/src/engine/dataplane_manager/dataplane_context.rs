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

use crate::engine::dataplane_drivers::DataplaneDriver;
use crate::engine::dataplane_manager::dataplane_commands::{
    DataplaneContinuation, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
};
use crate::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactoryTrait;
use crate::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxy, DataplaneProxyEgress, DataplaneProxyIngress,
};
use crate::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, NewDataplaneTransferDto, TransferRole, TransferState,
};

use crate::errors::DataplaneError;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use crate::DataplaneAddress;
use ymir::types::oauth::UserInfo;
use common::config::services::TransferConfig;
use ymir::config::traits::CommonConfigTrait;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use connector::{ConnectorInstanceDto, ConnectorInstanceFacadeTrait};
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[derive(Clone, Debug)]
pub struct DataplaneContext {
    config: Arc<TransferConfig>,
    dataplane_process: DataplaneTransferDto,
    connector_instance: Option<ConnectorInstanceDto>,
    driver: Option<DataplaneDriver>,
    runtime: Option<DataplaneRuntime>,
    proxy: Option<DataplaneProxy>,
    forward_dataplane_address: Option<DataplaneAddress>,
    transition_reason: Option<String>,
}

impl DataplaneContext {
    pub async fn from_init(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        _connector_entity: Arc<dyn ConnectorInstanceFacadeTrait>,
        config: Arc<TransferConfig>,
        init: DataplaneInitCommandTypes,
    ) -> Outcome<Self> {
        let id = Urn::from_str(&format!("urn:dataplane-transfer:{}", uuid::Uuid::new_v4()))?;
        let transfer_id = match &init {
            DataplaneInitCommandTypes::AsProvider {
                transfer_process_id,
                ..
            } => transfer_process_id,
            DataplaneInitCommandTypes::AsConsumer {
                transfer_process_id,
                ..
            } => transfer_process_id,
        };
        let transfer_role = match &init {
            DataplaneInitCommandTypes::AsProvider { .. } => TransferRole::Provider,
            DataplaneInitCommandTypes::AsConsumer { .. } => TransferRole::Consumer,
        };
        let interaction_mode = match &init {
            DataplaneInitCommandTypes::AsProvider { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { .. } => InteractionMode::Pull,
                DataplaneInitCommandDirection::Push { .. } => InteractionMode::Push,
            },
            DataplaneInitCommandTypes::AsConsumer { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { .. } => InteractionMode::Pull,
                DataplaneInitCommandDirection::Push { .. } => InteractionMode::Push,
            },
        };
        let connector_instance = match &init {
            DataplaneInitCommandTypes::AsProvider {
                connector_instance, ..
            } => Some(connector_instance.clone()),
            DataplaneInitCommandTypes::AsConsumer { .. } => None,
        };
        let data_plane_address = match &init {
            DataplaneInitCommandTypes::AsProvider { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { data_address } => data_address,
                DataplaneInitCommandDirection::Push { data_address } => data_address,
            },
            DataplaneInitCommandTypes::AsConsumer { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { data_address } => data_address,
                DataplaneInitCommandDirection::Push { data_address } => data_address,
            },
        };

        let init_reason = Self::describe_init(
            &transfer_role,
            &interaction_mode,
            transfer_id,
            connector_instance.as_ref().map(|c| &c.id),
            data_plane_address.as_ref(),
        );

        // db access
        let dataplane_process = dataplane_service
            .create(
                &Self::actor(),
                &NewDataplaneTransferDto {
                    owner: Some(init.owner().clone()),
                    id: Some(id),
                    transfer_process_id: transfer_id.to_string(),
                    role: transfer_role,
                    interaction_mode,
                    state: TransferState::Init,
                    connector_instance_id: connector_instance.as_ref().map(|c| c.id.clone()),
                    ingress_config: json!({}),
                    egress_config: json!({}),
                    reason: Some(init_reason),
                },
            )
            .await?;

        // context
        Ok(Self {
            config,
            dataplane_process,
            connector_instance,
            driver: None,
            runtime: None,
            proxy: None,
            forward_dataplane_address: data_plane_address.clone(),
            transition_reason: None,
        })
    }

    pub async fn from_continuation(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
        driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
        config: Arc<TransferConfig>,
        continuation: DataplaneContinuation,
        dataplane_address: Option<DataplaneAddress>,
    ) -> Outcome<Self> {
        // db access
        let dataplane_process = match dataplane_service
            .get_by_process_id(&Self::actor(), &continuation.transfer_dto_urn)
            .await
        {
            Err(Errors::MissingResourceError { .. }) => {
                return Err(DataplaneError::TransferNotFound {
                    transfer_process_id: continuation.transfer_dto_urn.to_string(),
                }
                .into())
            }
            other => other?,
        };
        // connector
        let connector_id = &dataplane_process.inner.connector_instance_id;
        let connector = match connector_id {
            Some(connector_id) => {
                let connector_urn = Urn::from_str(connector_id)?;
                connector_service
                    .get_instance_by_id(&connector_urn)
                    .await?
            }
            None => None,
        };

        // context
        let runtime = dataplane_process
            .clone()
            .inner
            .flow_control
            .and_then(|v| serde_json::from_value(v).ok());

        let mut context = Self {
            config,
            dataplane_process: dataplane_process.clone(),
            connector_instance: connector.clone(),
            driver: None,
            runtime,
            proxy: None,
            forward_dataplane_address: dataplane_address,
            transition_reason: continuation.reason,
        };

        // driver to context (both Provider and Consumer resolve their role/mode driver;
        // ConsumerPush in particular needs NoOpPubSubscriber on continuations)
        context.driver = Some(driver_factory.get_or_create_driver(&context)?);

        // proxy to context
        let ingress = serde_json::from_value::<DataplaneProxyIngress>(
            dataplane_process.inner.ingress_config.clone(),
        );
        let egress = serde_json::from_value::<DataplaneProxyEgress>(
            dataplane_process.inner.egress_config.clone(),
        );
        if let (Ok(ingress), Ok(egress)) = (ingress, egress) {
            if context.forward_dataplane_address.is_none() {
                context.forward_dataplane_address = Some(egress.clone().into());
            }
            context.proxy = Some(DataplaneProxy { ingress, egress });
        }

        Ok(context)
    }

    pub fn set_dataplane_process(&mut self, dataplane_process: DataplaneTransferDto) -> &mut Self {
        self.dataplane_process = dataplane_process;
        self
    }

    pub fn set_connector_instance(
        &mut self,
        connector_instance: ConnectorInstanceDto,
    ) -> &mut Self {
        self.connector_instance = Some(connector_instance);
        self
    }

    pub fn set_driver(&mut self, driver: DataplaneDriver) -> &mut Self {
        self.driver = Some(driver);
        self
    }

    pub fn set_runtime(&mut self, runtime: DataplaneRuntime) -> &mut Self {
        self.runtime = Some(runtime);
        self
    }

    pub fn set_proxy(&mut self, proxy: DataplaneProxy) -> &mut Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn set_forward_dataplane_address_from_ingress(&mut self) -> &mut Self {
        self.forward_dataplane_address = match &self.proxy {
            Some(proxy) => match proxy.ingress() {
                DataplaneProxyIngress::NoOp => None,
                DataplaneProxyIngress::HttpListener {
                    path,
                    token_type,
                    token,
                } => Some(DataplaneAddress {
                    endpoint_type: "HTTP".to_string(),
                    endpoint: self.forward_url(path),
                    authorization_type: token_type.clone(),
                    authorization: token.clone(),
                }),
            },
            None => None,
        };
        self
    }

    /// Public HTTP address of this dataplane for an ingress path.
    fn forward_url(&self, path: &str) -> String {
        let base = self.config.common().get_host(HostType::Http);
        format!("{base}{path}")
    }

    pub fn connector_instance(&self) -> Option<&ConnectorInstanceDto> {
        self.connector_instance.as_ref()
    }

    pub fn dataplane_process(&self) -> &DataplaneTransferDto {
        &self.dataplane_process
    }

    /// Who the engine acts as on this transfer's records: an in-process flow that already knows
    /// which process it wants, so no owner is checked (the process keeps its owner).
    pub fn engine_actor(&self) -> UserInfo {
        Self::actor()
    }

    fn actor() -> UserInfo {
        UserInfo::system()
    }

    pub fn dataplane_process_state(&self) -> TransferState {
        self.dataplane_process.inner.state.clone()
    }

    pub fn dataplane_process_interaction_mode(&self) -> InteractionMode {
        self.dataplane_process.inner.interaction_mode.clone()
    }

    pub fn dataplane_process_role(&self) -> TransferRole {
        self.dataplane_process.inner.role.clone()
    }

    pub fn forward_dataplane_address(&self) -> Option<&DataplaneAddress> {
        self.forward_dataplane_address.as_ref()
    }

    pub fn driver(&self) -> Option<&DataplaneDriver> {
        self.driver.as_ref()
    }
    pub fn runtime(&self) -> Option<&DataplaneRuntime> {
        self.runtime.as_ref()
    }

    pub fn proxy(&self) -> Option<&DataplaneProxy> {
        self.proxy.as_ref()
    }

    pub fn config(&self) -> Arc<TransferConfig> {
        self.config.clone()
    }

    pub fn transition_reason(&self) -> Option<&str> {
        self.transition_reason.as_deref()
    }

    pub fn take_transition_reason(&mut self) -> Option<String> {
        self.transition_reason.take()
    }

    pub fn set_transition_reason(&mut self, reason: Option<String>) -> &mut Self {
        self.transition_reason = reason;
        self
    }

    /// Strips query strings, fragments, and userinfo before logging endpoints.
    pub fn redact_endpoint(raw: &str) -> String {
        if raw.trim().is_empty() {
            return "<empty>".to_string();
        }
        match url::Url::parse(raw) {
            Ok(mut parsed) => {
                let _ = parsed.set_username("");
                let _ = parsed.set_password(None);
                parsed.set_query(None);
                parsed.set_fragment(None);
                parsed.to_string()
            }
            Err(_) => raw
                .split(['?', '#'])
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or("<redacted>")
                .to_string(),
        }
    }

    fn describe_init(
        role: &TransferRole,
        mode: &InteractionMode,
        transfer_id: &Urn,
        connector_id: Option<&Urn>,
        address: Option<&DataplaneAddress>,
    ) -> String {
        let mut msg = format!("Initialized {role} {mode} dataplane for transfer {transfer_id}");
        if let Some(id) = connector_id {
            msg.push_str(&format!(", connector={id}"));
        }
        if let Some(addr) = address.filter(|a| !a.endpoint.trim().is_empty()) {
            msg.push_str(&format!(
                ", endpoint={} ({})",
                Self::redact_endpoint(&addr.endpoint),
                addr.endpoint_type
            ));
        }
        msg
    }

    /// Human-readable, secret-free summary of the configured ingress endpoint.
    pub fn describe_ingress(&self) -> String {
        match self.proxy.as_ref().map(|p| &p.ingress) {
            Some(DataplaneProxyIngress::HttpListener { path, token, .. }) => {
                format!("HttpListener({path}, auth={})", token.is_some())
            }
            Some(DataplaneProxyIngress::NoOp) => "NoOp".to_string(),
            None => "Unconfigured".to_string(),
        }
    }

    /// Human-readable, secret-free summary of the configured egress target.
    pub fn describe_egress(&self) -> String {
        match self.proxy.as_ref().map(|p| &p.egress) {
            Some(DataplaneProxyEgress::HttpProxy { path, token, .. }) => {
                format!(
                    "HttpProxy({}, auth={})",
                    Self::redact_endpoint(path),
                    token.is_some()
                )
            }
            Some(DataplaneProxyEgress::DataClient { path, .. }) => {
                format!("DataClient({})", Self::redact_endpoint(path))
            }
            Some(DataplaneProxyEgress::NoOp) => "NoOp".to_string(),
            None => "Unconfigured".to_string(),
        }
    }

    /// Human-readable, secret-free summary of resolved runtime authentication.
    pub fn describe_auth(&self) -> String {
        match self.runtime.as_ref().map(|rt| &rt.auth) {
            Some(ResolvedAuthCredentials::NoAuth) | None => "NoAuth".to_string(),
            Some(ResolvedAuthCredentials::BearerToken { .. }) => "BearerToken".to_string(),
            Some(ResolvedAuthCredentials::ApiKey { key, location, .. }) => {
                format!("ApiKey(key={key}, location={location:?})")
            }
            Some(ResolvedAuthCredentials::BasicAuth { .. }) => "BasicAuth".to_string(),
            Some(ResolvedAuthCredentials::OAuth2 {
                token_type,
                expires_at,
                ..
            }) => format!(
                "OAuth2(token_type={token_type}, has_expiry={})",
                expires_at.is_some()
            ),
        }
    }
}
