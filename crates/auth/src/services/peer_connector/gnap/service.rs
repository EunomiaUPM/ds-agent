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

use std::sync::Arc;

use super::super::PeerConnectorTrait;
use crate::services::peer_connector::gnap::config::GnapPeerConnectorConfig;
use crate::types::entities::ReachProvider;
use crate::types::response::{RotationOutcome, TokenWhatResponse};
use async_trait::async_trait;
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use chrono::{DateTime, Duration, Utc};
use common::config::types::traits::EntityClientTrait;
use common::routes::auth::{gate, peer_connection};
use common::utils::parse_url;
use serde_json::Value;
use tracing::info;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::{participant, participant_relation, resource_req};
use ymir::errors::{Errors, Outcome};
use ymir::http::routes::{fill, wallet};
use ymir::services::client::{ClientExt, ClientTrait};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::gnap::access_token::AccessToken;
use ymir::types::gnap::grant_request::access::AccessType;
use ymir::types::gnap::grant_request::interact::{FinishMethod, InteractAction, InteractStart};
use ymir::types::gnap::grant_request::{GrantKind, GrantRequest};
use ymir::types::gnap::grant_response::{ErrorCode, GrantResponse, GrantResponseKind};
use ymir::types::gnap::GrantStatus;
use ymir::types::http::HttpBody;
use ymir::types::keys::{Certificate, KeySource, PrivateKey};
use ymir::types::oauth::UserInfo;
use ymir::types::participants::ParticipantType;
use ymir::types::secrets::{PemHelper, StringHelper};
use ymir::utils::{
    expect_from_env, get_query_param, http_client, json_headers, require_field, trim_4_base,
    ParseHeaderExt, ResponseExt,
};

/// GNAP client towards peers; reads the agent's certificate from the vault.
pub struct GnapPeerConnectorService {
    vault: Arc<VaultService>,
    config: GnapPeerConnectorConfig,
}

impl GnapPeerConnectorService {
    pub fn new(
        vault: Arc<VaultService>,
        config: GnapPeerConnectorConfig,
    ) -> GnapPeerConnectorService {
        GnapPeerConnectorService { vault, config }
    }

    async fn signing_material(&self) -> Outcome<(String, KeySource, PrivateKey)> {
        let cert = expect_from_env("VAULT_APP_CERT");
        let cert: StringHelper = self.vault.read(None, &cert).await?;
        let certificate = Certificate::try_from_pem(cert.data())?;
        let key_source = KeySource::Cert(certificate);

        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: PemHelper = self.vault.read(None, &priv_key).await?;
        let priv_key = PrivateKey::from_safe_pem(priv_key.pem(), priv_key.kty(), priv_key.crv())?;

        Ok((cert.data().to_string(), key_source, priv_key))
    }

    async fn managing_headers(&self, grant: &grant::Model, method: &str) -> Outcome<HeaderMap> {
        let uri = require_field(grant.managing_uri.as_ref(), "managing uri")?;
        let token = require_field(grant.managing_token.as_ref(), "managing token")?;
        let (_, key_source, priv_key) = self.signing_material().await?;

        let authorization = format!("GNAP {}", token);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization.parse_header()?);
        let httpsig = HttpSig::build(
            &key_source,
            &priv_key,
            None,
            method,
            uri,
            &[],
            "application/json",
            Some(&authorization),
        )?;
        headers.extend(httpsig);
        Ok(headers)
    }

    fn expiry(now: DateTime<Utc>, expires_in: Option<u64>) -> Option<DateTime<Utc>> {
        expires_in.map(|secs| now + Duration::seconds(secs as i64))
    }

    fn apply_access_token(grant: &mut grant::Model, access_token: AccessToken) {
        let now = Utc::now();
        grant.final_token = Some(access_token.value);
        grant.final_expires_at = Self::expiry(now, access_token.expires_in);
        match access_token.manage {
            Some(manage) => {
                grant.managing_uri = Some(manage.uri);
                grant.managing_token = Some(manage.access_token.value);
                grant.managing_expires_at = Self::expiry(now, manage.access_token.expires_in);
            }
            None => {
                grant.managing_uri = None;
                grant.managing_token = None;
                grant.managing_expires_at = None;
            }
        }
    }
}

#[async_trait]
impl PeerConnectorTrait for GnapPeerConnectorService {
    fn build_grant_plan(&self, user_info: &UserInfo, payload: ReachProvider) -> grant::Plan {
        grant::Plan {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_info.id().to_string(),
            username: user_info.username().map(ToString::to_string),
            role: user_info.role().clone(),
            participant_id: payload.id,
            participant_nick: payload.nick,
            visibility: payload.visibility,
            vc_type_config: None,
            grant_endpoint: payload.url,
            auto: Some(payload.auto.unwrap_or(true)),
            requested: payload.requested.unwrap_or(true),
            kind: GrantKind::AccessToken,
        }
    }
    fn build_interaction_plan(&self, id: &str) -> interaction::Plan {
        let callback_uri = format!(
            "{}{}{}{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
            peer_connection::PREFIX,
            fill(peer_connection::CALLBACK, id)
        );

        interaction::Plan {
            id: id.to_string(),
            start: vec![InteractStart::Oid4VP],
            method: FinishMethod::Push,
            callback_uri,
            hash_method: None,
            hints: None,
        }
    }
    fn build_resource_req_plan(
        &self,
        id: &str,
        actions: Vec<InteractAction>,
    ) -> resource_req::Model {
        let mut actions: Vec<InteractAction> = actions
            .into_iter()
            .filter(|a| !matches!(a, InteractAction::Other(_) | InteractAction::RequestVc))
            .collect();

        if actions.is_empty() {
            actions.push(InteractAction::Talk);
        }

        resource_req::Model {
            id: id.to_string(),
            r#type: AccessType::ApiAccess,
            actions,
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None,
        }
    }

    fn build_verification_plan(&self, uri: &str, id: &str) -> Outcome<verification::Plan> {
        info!("Saving verification data");

        // url::Url doesn't accept custom schemes; rewrite to https just for parsing.
        let fixed_uri = uri.replacen("openid4vp://", "https://", 1);
        let parsed_uri = parse_url(&fixed_uri)?;

        let response_type = get_query_param(&parsed_uri, "response_type")?;
        let client_id = get_query_param(&parsed_uri, "client_id")?;
        let response_mode = get_query_param(&parsed_uri, "response_mode")?;
        let pd_uri = get_query_param(&parsed_uri, "presentation_definition_uri")?;
        let client_id_scheme = get_query_param(&parsed_uri, "client_id_scheme")?;
        let nonce = get_query_param(&parsed_uri, "nonce")?;
        let response_uri = get_query_param(&parsed_uri, "response_uri")?;

        Ok(verification::Plan {
            id: id.to_string(),
            uri: uri.to_string(),
            scheme: "openid4vp".to_string(),
            response_type,
            client_id,
            response_mode,
            pd_uri,
            client_id_scheme,
            nonce,
            response_uri,
        })
    }

    fn build_mate_plan(&self, grant: &grant::Model) -> participant::Plan {
        let base_url = trim_4_base(&grant.grant_endpoint);
        participant::Plan {
            participant_id: grant.participant_id.clone(),
            participant_nick: grant.participant_nick.clone(),
            participant_type: ParticipantType::Agent,
            base_url,
            extra_fields: None,
        }
    }

    fn build_mate_relation(&self, grant: &grant::Model) -> participant_relation::Model {
        participant_relation::Model {
            user_id: grant.user_id.clone(),
            username: grant.username.clone(),
            participant_id: grant.participant_id.clone(),
            role: grant.role.clone(),
            visibility: grant.visibility.clone(),
        }
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
        resource_req: &resource_req::Model,
    ) -> Outcome<GrantResponse> {
        info!("Sending request to establish connection with peer");

        let (cert, key_source, priv_key) = self.signing_material().await?;

        let client = self.config.get_client(&cert)?;

        let grant_request =
            GrantRequest::new_token(client, resource_req.actions.clone(), interaction);

        let (body, body_bytes) = HttpBody::from_json_bytes(&grant_request)?;

        let mut headers = json_headers();
        let httpsig = HttpSig::build(
            &key_source,
            &priv_key,
            None,
            "POST",
            &grant.grant_endpoint,
            &body_bytes,
            "application/json",
            None,
        )?;

        headers.extend(httpsig);

        let res = http_client()
            .post(&grant.grant_endpoint, Some(headers), body)
            .await?;

        res.parse_json().await
    }

    fn manage_grant_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
        interaction: &mut interaction::Model,
    ) -> Outcome<TokenWhatResponse> {
        match response {
            GrantResponse::Approved(payload) => match payload.kind {
                GrantResponseKind::AccessToken { access_token } => {
                    grant.status = GrantStatus::Approved;
                    Self::apply_access_token(grant, access_token);
                    Ok(TokenWhatResponse::Completed)
                }
                GrantResponseKind::CredentialResponse { .. } => Err(Errors::provider_grant(
                    "Provider returned an OID4VCI URI when asking for a token",
                )),
            },
            GrantResponse::Pending(payload) => {
                grant.status = GrantStatus::Pending;
                grant.as_assigned_id = payload.instance_id;

                interaction.as_nonce = payload.interact.finish;
                interaction.oidc_vp_uri = payload.interact.oid4vp.clone();
                interaction.continuation_token = Some(payload.r#continue.access_token.value);
                interaction.continuation_endpoint = Some(payload.r#continue.uri);
                interaction.continuation_wait = payload.r#continue.wait.map(|n| n as i64);
                let uri = payload.interact.oid4vp.ok_or_else(|| {
                    Errors::provider_grant(
                        "Provider did not send expected interaction method (oid4vp)",
                    )
                })?;
                Ok(TokenWhatResponse::Presentation(uri))
            }
            GrantResponse::Processing(..) => {
                grant.status = GrantStatus::Processing;
                Ok(TokenWhatResponse::Wait)
            }
            GrantResponse::Error(error) => {
                grant.status = GrantStatus::Rejected;
                grant.ended_at = Some(chrono::Utc::now());
                Err(Errors::provider_grant(format!(
                    "Provider said {}",
                    error.error
                )))
            }
        }
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn send_rotation_req(&self, grant: &grant::Model) -> Outcome<GrantResponse> {
        info!("Rotating the token of a grant with a peer");
        let uri = require_field(grant.managing_uri.as_ref(), "managing uri")?;
        let headers = self.managing_headers(grant, "POST").await?;
        let res = http_client()
            .post(uri, Some(headers), HttpBody::None)
            .await?;
        res.parse_json().await
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn send_revocation_req(&self, grant: &grant::Model) -> Outcome<()> {
        info!("Revoking the token of a grant with a peer");
        let uri = require_field(grant.managing_uri.as_ref(), "managing uri")?;
        let headers = self.managing_headers(grant, "DELETE").await?;
        http_client()
            .delete(uri, Some(headers), HttpBody::None)
            .await?
            .ensure_success()
            .await?;
        Ok(())
    }

    fn apply_rotation_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
    ) -> Outcome<RotationOutcome> {
        match response {
            GrantResponse::Approved(payload) => match payload.kind {
                GrantResponseKind::AccessToken { access_token } => {
                    Self::apply_access_token(grant, access_token);
                    Ok(RotationOutcome::Rotated)
                }
                GrantResponseKind::CredentialResponse { .. } => Err(Errors::provider_grant(
                    "Provider returned an OID4VCI URI when rotating a token",
                )),
            },
            GrantResponse::Error(error) => match error.error {
                ErrorCode::InvalidRotation => Ok(RotationOutcome::Refused),
                other => Err(Errors::provider_grant(format!("Provider said {}", other))),
            },
            _ => Err(Errors::provider_grant(
                "Provider answered a token rotation with an unexpected response",
            )),
        }
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn discover_gate(&self, base_url: &str) -> Outcome<String> {
        let base_url = base_url.trim_end_matches('/');
        let did_doc: Value = http_client()
            .get_json(&format!("{base_url}{}", wallet::DID_DOC), None)
            .await?;
        let advertised = did_doc
            .get("service")
            .and_then(Value::as_array)
            .and_then(|services| {
                services.iter().find(|service| {
                    service.get("type").and_then(Value::as_str) == Some("AuthorizationServer")
                })
            })
            .and_then(|service| service.get("serviceEndpoint"))
            .and_then(Value::as_str)
            .map(|endpoint| endpoint.trim_end_matches('/').to_string());
        let gate_url = match advertised {
            Some(gate_url) => gate_url,
            None => format!("{base_url}{}{}", self.config.get_api_path(), gate::PREFIX),
        };
        Ok(format!("{gate_url}{}", gate::ACCESS))
    }
}
