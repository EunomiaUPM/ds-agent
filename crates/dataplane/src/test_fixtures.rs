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

/// Test helpers shared across authentication unit tests.
///
/// Each function returns a `DataplaneContext` wired with a specific auth config
/// so that individual authenticator tests can focus on behaviour rather than
/// setup boilerplate.
use crate::data::entities::dataplane_transfers;
use crate::entities::dataplane_manager::dataplane_commands::{
    DataplaneInitCommandDirection, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, MockDataplaneTransfersEntitiesTrait, TransferRole,
    TransferState,
};
use crate::DataplaneAddress;
use common::test_utils::config_fixtures::transfer_config_fixture;
use connector::{
    ApiKeyLocation, AuthenticationConfig, BasicAuthConfig, ConnectorInstanceDto,
    ConnectorInstanceTrait, ConnectorInstantiationDto, ConnectorMetadata, HttpSpec,
    InteractionConfig, OAuthGrantType, ProtocolSpec, PullLifecycle, SecretSource, SecretString,
    TemplateVecString,
};
use mockall::mock;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

// local mock for ConnectorInstanceTrait ────────────────────────────────────

mock! {
    pub ConnectorInstance {}
    #[async_trait::async_trait]
    impl ConnectorInstanceTrait for ConnectorInstance {
        async fn get_instance_by_id(&self, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>>;
        async fn get_instance_by_distribution(
            &self,
            distribution_id: &Urn,
        ) -> Outcome<Option<ConnectorInstanceDto>>;
        async fn upsert_instance(
            &self,
            dto: &mut ConnectorInstantiationDto,
        ) -> Outcome<ConnectorInstanceDto>;
        async fn delete_instance_by_id(&self, id: &Urn) -> Outcome<()>;
    }
}

// internal helpers ──────────────────────────────────────────────────────────

fn tp_urn() -> Urn {
    Urn::from_str("urn:transfer-process:test-1").unwrap()
}

pub(crate) fn plain_secret(value: &str) -> SecretString {
    SecretString {
        source: SecretSource::Plain(value.to_string()),
    }
}

fn dummy_interaction() -> InteractionConfig {
    InteractionConfig::Pull(PullLifecycle {
        data_access: ProtocolSpec::Http(HttpSpec {
            url_template: "https://example.com/data".to_string(),
            method: TemplateVecString::Value(vec!["GET".to_string()]),
            headers: None,
            body_template: None,
        }),
    })
}

fn dummy_connector(auth: AuthenticationConfig) -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: Urn::from_str("urn:connector:test-1").unwrap(),
        metadata: ConnectorMetadata {
            name: Some("test".to_string()),
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication_config: auth,
        interaction: dummy_interaction(),
        distribution_id: Urn::from_str("urn:distribution:test-1").unwrap(),
    }
}

fn provider_dto(state: TransferState) -> DataplaneTransferDto {
    DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            id: "urn:dataplane-transfer:test-1".to_string(),
            transfer_process_id: tp_urn().to_string(),
            role: TransferRole::Provider,
            interaction_mode: InteractionMode::Pull,
            state,
            connector_instance_id: Some("urn:connector:test-1".to_string()),
            ingress_config: json!({}),
            egress_config: json!({}),
            flow_control: None,
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }
}

fn consumer_dto(state: TransferState) -> DataplaneTransferDto {
    let mut dto = provider_dto(state);
    dto.inner.role = TransferRole::Consumer;
    dto.inner.connector_instance_id = None;
    dto
}

fn forward_address() -> DataplaneAddress {
    DataplaneAddress {
        endpoint_type: "HTTP".to_string(),
        endpoint: "http://provider.example.com/data".to_string(),
        authorization_type: None,
        authorization: None,
    }
}

async fn provider_context(auth: AuthenticationConfig) -> DataplaneContext {
    let mut mock = MockDataplaneTransfersEntitiesTrait::new();
    mock.expect_create_dataplane_transfer()
        .returning(|_| Ok(provider_dto(TransferState::Init)));
    let connector = dummy_connector(auth);
    DataplaneContext::from_init(
        Arc::new(mock),
        Arc::new(MockConnectorInstance::new()),
        transfer_config_fixture(),
        DataplaneInitCommandTypes::AsProvider {
            transfer_process_id: tp_urn(),
            connector_instance: connector,
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(forward_address()),
            },
        },
    )
    .await
    .unwrap()
}

// public fixtures ───────────────────────────────────────────────────────────

/// Provider context with `NoAuth` — useful to confirm wrong-type rejections.
pub async fn no_auth_context() -> DataplaneContext {
    provider_context(AuthenticationConfig::NoAuth).await
}

/// Provider context with a plain-text Bearer token.
pub async fn bearer_context(token: &str) -> DataplaneContext {
    provider_context(AuthenticationConfig::BearerToken {
        token: plain_secret(token),
    })
    .await
}

/// Provider context with a plain-text API key.
pub async fn api_key_context(key: &str, value: &str, location: ApiKeyLocation) -> DataplaneContext {
    provider_context(AuthenticationConfig::ApiKey {
        key: key.to_string(),
        value: plain_secret(value),
        location,
    })
    .await
}

/// Provider context with plain-text HTTP Basic Auth credentials.
pub async fn basic_auth_context(username: &str, password: &str) -> DataplaneContext {
    provider_context(AuthenticationConfig::BasicAuth(BasicAuthConfig {
        username: username.to_string(),
        password: plain_secret(password),
    }))
    .await
}

/// Provider context with OAuth2 client credentials (plain-text secret).
pub async fn oauth2_context(
    token_url: &str,
    client_id: &str,
    client_secret: &str,
) -> DataplaneContext {
    provider_context(AuthenticationConfig::OAuth2 {
        grant_type: OAuthGrantType::ClientCredentials,
        token_url: token_url.to_string(),
        client_id: client_id.to_string(),
        client_secret: plain_secret(client_secret),
        scopes: TemplateVecString::Value(vec![]),
        on_token_expire: Default::default(),
    })
    .await
}

/// Provider context with OAuth2 Resource Owner Password Credentials (plain-text secrets).
pub async fn oauth2_password_context(
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
) -> DataplaneContext {
    provider_context(AuthenticationConfig::OAuth2 {
        grant_type: OAuthGrantType::Password {
            username: username.to_string(),
            password: plain_secret(password),
        },
        token_url: token_url.to_string(),
        client_id: client_id.to_string(),
        client_secret: plain_secret(client_secret),
        scopes: TemplateVecString::Value(vec![]),
        on_token_expire: Default::default(),
    })
    .await
}

/// Consumer pull context — no connector instance. Authenticators that require a
/// connector instance must return an error for this context.
pub async fn consumer_context() -> DataplaneContext {
    let mut mock = MockDataplaneTransfersEntitiesTrait::new();
    mock.expect_create_dataplane_transfer()
        .returning(|_| Ok(consumer_dto(TransferState::Init)));
    DataplaneContext::from_init(
        Arc::new(mock),
        Arc::new(MockConnectorInstance::new()),
        transfer_config_fixture(),
        DataplaneInitCommandTypes::AsConsumer {
            transfer_process_id: tp_urn(),
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(forward_address()),
            },
        },
    )
    .await
    .unwrap()
}
