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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use common::config::types::traits::EntityClientTrait;
use reqwest::Response;
use tracing::info;
use url::Url;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::participant;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::services::client::ClientTrait;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::gnap::grant_request::interact::{FinishMethod, InteractStart};
use ymir::types::gnap::grant_request::{GrantKind, GrantRequest};
use ymir::types::gnap::grant_response::{ErrorCode, GrantResponse, GrantResponseKind};
use ymir::types::gnap::GrantStatus;
use ymir::types::http::HttpBody;
use ymir::types::keys::{Certificate, KeySource, PrivateKey};
use ymir::types::participants::ParticipantType;
use ymir::types::secrets::{PemHelper, StringHelper};
use ymir::types::vcs::{VcType, VcTypeConfig};
use ymir::utils::{
    expect_from_env, get_query_param, http_client, json_headers, require_field, trim_4_base,
    ResponseExt,
};

use super::super::VcRequesterTrait;
use super::config::VCRequesterConfig;
use crate::types::entities::ReachAuthority;
use crate::types::response::VcWhatResponse;
use crate::utils::parse_url;

pub struct VCReqService {
    vault: Arc<VaultService>,
    config: VCRequesterConfig,
}

impl VCReqService {
    pub fn new(vault: Arc<VaultService>, config: VCRequesterConfig) -> Self {
        VCReqService { config, vault }
    }
}

#[async_trait]
impl VcRequesterTrait for VCReqService {
    fn build_grant_plan(&self, payload: ReachAuthority) -> grant::Plan {
        grant::Plan {
            id: uuid::Uuid::new_v4().to_string(),
            participant_id: payload.id,
            participant_nick: payload.nick,
            grant_endpoint: payload.url,
            vc_type_config: Some(vec![payload.vc_type]),
            auto: payload.auto,
            kind: GrantKind::CredentialRequest,
        }
    }
    fn build_interaction_plan(&self, id: &str, start: InteractStart) -> interaction::Plan {
        let callback_uri = format!(
            "{}{}/vc-request/callback/{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
            &id
        );

        let start = match start {
            InteractStart::Other(_) => InteractStart::Other(String::new()),
            other => other,
        };

        interaction::Plan {
            id: id.to_string(),
            start: vec![start],
            method: FinishMethod::Push,
            callback_uri,
            hash_method: None,
            hints: None,
        }
    }

    fn build_verification_plan(&self, uri: &str, id: &str) -> Outcome<verification::Plan> {
        info!("Saving verification data");

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

    fn build_authority_plan(&self, grant: &grant::Model) -> participant::Plan {
        let base_url = trim_4_base(&grant.grant_endpoint);
        participant::Plan {
            participant_id: grant.participant_id.clone(),
            participant_nick: grant.participant_nick.clone(),
            participant_type: ParticipantType::Authority,
            base_url,
            token: None,
            extra_fields: None,
            is_me: false,
        }
    }

    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
    ) -> Outcome<GrantResponse> {
        info!("Sending grant request request to authority");

        let vc_type_config = require_field(grant.vc_type_config, "vc_type_config")?;
        let cert = expect_from_env("VAULT_APP_CERT");
        let cert: StringHelper = self.vault.read(None, &cert).await?;
        let certificate = Certificate::try_from_pem(cert.data())?;
        let key_source = KeySource::Cert(certificate);

        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: PemHelper = self.vault.read(None, &priv_key).await?;
        let priv_key = PrivateKey::from_safe_pem(priv_key.pem(), priv_key.kty(), priv_key.crv())?;

        let client = self.config.get_client(cert.data())?;

        let grant_request = GrantRequest::new_vc(client, vc_type_config, interaction);

        let (body, body_bytes) = HttpBody::from_json_bytes(&grant_request)?;

        let mut headers = json_headers();
        let httpsig = HttpSig::build(
            &key_source,
            &priv_key,
            None,
            "POST",
            &grant.grant_endpoint,
            &body_bytes,
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
    ) -> Outcome<VcWhatResponse> {
        match response {
            GrantResponse::Approved(payload) => match payload.kind {
                GrantResponseKind::AccessToken { .. } => Err(Errors::authority_grant(
                    "Authority returned a token when asking for a OID4VCI URI ",
                )),
                GrantResponseKind::CredentialResponse {
                    credential_response,
                } => {
                    grant.status = GrantStatus::Approved;
                    grant.ended_at = Some(chrono::Utc::now());
                    Ok(VcWhatResponse::Issuance(credential_response.credential_uri))
                }
            },

            GrantResponse::Pending(payload) => {
                grant.status = GrantStatus::Pending;
                grant.as_assigned_id = payload.instance_id;

                interaction.as_nonce = payload.interact.finish;
                interaction.oidc_vp_uri = payload.interact.oid4vp.clone();
                interaction.continue_token = Some(payload.r#continue.access_token.value);
                interaction.continue_endpoint = Some(payload.r#continue.uri);
                interaction.continue_wait = payload.r#continue.wait.map(|n| n as i64);
                let uri = payload.interact.oid4vp.ok_or_else(|| {
                    Errors::authority_grant(
                        "Authority did not send expected interaction method (oid4vp)",
                    )
                })?;
                Ok(VcWhatResponse::Presentation(uri))
            }
            GrantResponse::Processing(..) => {
                grant.status = GrantStatus::Processing;
                Ok(VcWhatResponse::Wait)
            }
            GrantResponse::Error(error) => {
                grant.status = GrantStatus::Rejected;
                grant.ended_at = Some(chrono::Utc::now());
                Err(Errors::authority_grant(format!(
                    "Authority said {}",
                    error.error
                )))
            }
        }
    }
}
