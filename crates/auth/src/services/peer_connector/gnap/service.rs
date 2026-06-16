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

use std::sync::Arc;

use super::super::PeerConnectorTrait;
use crate::services::peer_connector::gnap::config::GnapPeerConnectorConfig;
use crate::types::entities::ReachProvider;
use crate::types::plan::PeerConnectionPlan;
use crate::utils::parse_url;
use async_trait::async_trait;
use common::config::types::traits::EntityClientTrait;
use reqwest::{Client, RequestBuilder, Response};
use tracing::info;
use url::Url;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::{participant, resource_req};
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientTrait;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::gnap::access_token::AccessToken;
use ymir::types::gnap::grant_request::access::AccessType;
use ymir::types::gnap::grant_request::interact::{FinishMethod, InteractAction, InteractStart};
use ymir::types::gnap::grant_request::{GrantKind, GrantRequest, GrantRequestKind};
use ymir::types::gnap::grant_response::{GrantResponse, GrantResponseKind};
use ymir::types::gnap::GrantStatus;
use ymir::types::http::HttpBody;
use ymir::types::keys::{Certificate, PrivateKey};
use ymir::types::participants::ParticipantType;
use ymir::types::secrets::{PemHelper, StringHelper};
use ymir::utils::{expect_from_env, get_query_param, json_headers, trim_4_base, ResponseExt};

pub struct GnapPeerConnectorService {
    client: Arc<dyn ClientTrait>,
    vault: Arc<VaultService>,
    config: GnapPeerConnectorConfig,
}

impl GnapPeerConnectorService {
    pub fn new(
        client: Arc<dyn ClientTrait>,
        vault: Arc<VaultService>,
        config: GnapPeerConnectorConfig,
    ) -> GnapPeerConnectorService {
        GnapPeerConnectorService {
            client,
            vault,
            config,
        }
    }
}

#[async_trait]
impl PeerConnectorTrait for GnapPeerConnectorService {
    fn build_peer_plan(&self, payload: &ReachProvider) -> PeerConnectionPlan {
        info!("Preparing plan to connect to peer");

        let id = uuid::Uuid::new_v4().to_string();
        let callback_uri = format!(
            "{}{}/peer-connection/callback/{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
            &id
        );

        let grant = grant::Plan {
            id: id.clone(),
            participant_id: payload.id.clone(),
            participant_nick: payload.slug.clone(),
            vc_type_config: None,
            grant_endpoint: payload.url.clone(),
            auto: payload.auto,
            kind: GrantKind::AccessToken,
        };

        let interaction = interaction::Plan {
            id: id.clone(),
            start: vec![InteractStart::Oid4VP],
            method: FinishMethod::Push,
            callback_uri: callback_uri.clone(),
            hash_method: None,
            hints: None,
        };

        let mut actions: Vec<InteractAction> = payload
            .actions
            .into_iter()
            .filter(|a| match a {
                InteractAction::Other(_) => false,
                InteractAction::RequestVc => false,
                _ => true,
            })
            .collect();
        if actions.is_empty() {
            actions.push(InteractAction::Talk);
        }

        let resource_req = resource_req::Model {
            id,
            r#type: AccessType::ApiAccess,
            actions,
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None,
        };

        PeerConnectionPlan {
            grant,
            interaction,
            resource_req,
        }
    }

    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
        resource_req: &resource_req::Model,
    ) -> Outcome<GrantResponse> {
        info!("Sending request to establish connection with peer");

        let cert = expect_from_env("VAULT_APP_CERT");
        let cert: StringHelper = self.vault.read(None, &cert).await?;
        let certificate = Certificate::try_from_pem(cert.data())?;

        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: PemHelper = self.vault.read(None, &priv_key).await?;
        let priv_key = PrivateKey::from_safe_pem(priv_key.pem(), priv_key.kty(), priv_key.crv())?;

        let client = self.config.get_client(cert.data())?;

        let grant_request =
            GrantRequest::new_token(client, resource_req.actions.clone(), interaction);

        let (body, body_bytes) = HttpBody::from_json_bytes(&grant_request)?;

        let mut headers = json_headers();
        let httpsig = HttpSig::build(
            &certificate,
            &priv_key,
            None,
            "POST",
            &grant.grant_endpoint,
            &body_bytes,
            None,
        )?;

        headers.extend(httpsig);

        let res = self
            .client
            .post(&grant.grant_endpoint, Some(headers), body)
            .await?;

        res.parse_json().await
    }

    fn manage_grant_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
        interaction: &mut interaction::Model,
    ) -> Outcome<String> {
        match response {
            GrantResponse::Approved(payload) => {
                // grant.status = GrantStatus::Approved; TODO
                match payload.kind {
                    GrantResponseKind::AccessToken { .. } => Err(Errors::provider_grant(
                        "Provider returned a token before expected",
                    )),
                    GrantResponseKind::CredentialResponse { .. } => Err(Errors::provider_grant(
                        "Provider returned an OID4VCI URI when asking for a token",
                    )),
                }
            }
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
                Ok(uri)
            }
            GrantResponse::Processing(..) => Err(Errors::provider_grant(
                "Provider needs to take more time to process request (unexpected)",
            )),
            GrantResponse::Error(error) => Err(Errors::provider_grant(format!(
                "Provider said {}",
                error.error
            ))),
        }
    }

    fn build_verification_plan(&self, id: &str, uri: &str) -> Outcome<verification::Plan> {
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

    fn manage_cont_resp(&self, response: GrantResponse, grant: &mut grant::Model) -> Outcome<()> {
        match response {
            GrantResponse::Approved(payload) => match payload.kind {
                GrantResponseKind::AccessToken { access_token } => {
                    grant.status = GrantStatus::Approved;
                    grant.token = Some(access_token.value);
                    Ok(())
                }
                GrantResponseKind::CredentialResponse { .. } => Err(Errors::provider_grant(
                    "Provider returned an OID4VCI URI when asking for a token",
                )),
            },
            GrantResponse::Pending(..) => Err(Errors::provider_grant(
                "Provider expected more verification, when I did not",
            )),
            GrantResponse::Processing(..) => Err(Errors::provider_grant(
                "Provider needs to take more time to process request (unexpected)",
            )),
            GrantResponse::Error(error) => Err(Errors::provider_grant(format!(
                "Provider said {}",
                error.error
            ))),
        }
    }

    fn build_mate_plan(&self, grant: &grant::Model) -> participant::Plan {
        let base_url = trim_4_base(&grant.grant_endpoint);
        participant::Plan {
            participant_id: grant.participant_id.clone(),
            participant_nick: grant.participant_nick.clone(),
            participant_type: ParticipantType::Agent,
            base_url,
            token: grant.token.clone(),
            extra_fields: None,
            is_me: false,
        }
    }
    fn manage_rejection(&self, model: &mut grant::Model) {
        info!("Managing rejection");
        model.status = GrantStatus::Rejected;
        model.ended_at = Some(chrono::Utc::now());
    }
}
