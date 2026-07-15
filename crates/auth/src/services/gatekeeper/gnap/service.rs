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

use super::super::GateKeeperTrait;
use super::config::GnapGateKeeperConfig;
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use tracing::info;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::received::{grant, interaction};
use ymir::data::entities::shared::{participant, resource_req};
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::services::client::ClientTrait;
use ymir::types::gnap::grant_request::client::{Client, KeyMaterial, KeyProof};
use ymir::types::gnap::grant_request::interact::{
    FinishMethod, HashMethod, InteractAction, InteractRequest, InteractStart,
};
use ymir::types::gnap::grant_request::{GrantKind, GrantRequest, GrantRequestKind};
use ymir::types::gnap::{
    ApprovedCallbackBody, CallbackBody, ContinueRequest, InteractionFinishResponse,
    RejectedCallbackBody,
};
use ymir::types::http::HttpBody;
use ymir::types::keys::{Certificate, DbKeySource, KeySource, PublicKey};
use ymir::types::participants::ParticipantType;
use ymir::utils::{
    create_opaque_token, extract_gnap_token, http_client, json_headers, trim_4_base,
};

pub struct GnapGateKeeperService {
    config: GnapGateKeeperConfig,
}

impl GnapGateKeeperService {
    pub fn new(config: GnapGateKeeperConfig) -> GnapGateKeeperService {
        GnapGateKeeperService { config }
    }
}

#[async_trait]
impl GateKeeperTrait for GnapGateKeeperService {
    fn build_grant_plan(&self, class_id: Option<String>) -> Outcome<grant::Plan> {
        let class_id = class_id.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Missing field class_id (used for nick) in the petition",
                None,
            )
        })?;

        let id = uuid::Uuid::new_v4().to_string();

        Ok(grant::Plan {
            id: id.clone(),
            participant_nick: class_id,
            vc_type_config: None,
            kind: GrantKind::AccessToken,
        })
    }
    fn build_resource_req_plan(
        &self,
        id: &str,
        grant_request_kind: GrantRequestKind,
    ) -> Outcome<resource_req::Model> {
        let access_req = match grant_request_kind {
            GrantRequestKind::AccessToken { access_token } => access_token,
            GrantRequestKind::CredentialRequest { .. } => {
                return Err(Errors::format(
                    BadFormat::Received,
                    "Unable to issue credentials, just tokens",
                    None,
                ))
            }
        };

        let mut actions: Vec<InteractAction> = access_req
            .access
            .actions
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|a| !matches!(a, InteractAction::Other(_) | InteractAction::RequestVc))
            .collect();
        if actions.is_empty() {
            actions.push(InteractAction::Talk);
        }

        let resource_req = resource_req::Model {
            id: id.to_string(),
            r#type: access_req.access.r#type,
            actions,
            locations: access_req.access.locations,
            datatypes: access_req.access.datatypes,
            identifier: access_req.access.identifier,
            privileges: access_req.access.privileges,
            label: access_req.label,
            flags: access_req.flags,
        };

        Ok(resource_req)
    }

    fn build_interaction_plan(
        &self,
        id: &str,
        client: Client,
        interact: Option<InteractRequest>,
    ) -> Outcome<interaction::Plan> {
        info!("Managing Grant Request");

        let interact = interact.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Petition malformed, interact field expected",
                None,
            )
        })?;

        if !interact.start.contains(&InteractStart::Oid4VP) {
            return Err(Errors::format(
                BadFormat::Received,
                "Expected interact method oid4vp",
                None,
            ));
        }

        let finish = interact.finish.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Petition malformed. Expected inclusion of finish indicator in request",
                None,
            )
        })?;

        let method = match finish.method {
            FinishMethod::Other(other) => {
                return Err(Errors::not_impl(
                    format!("Interact method {other} not supported"),
                    None,
                ))
            }
            supported => supported,
        };

        let callback_uri = finish.uri.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Petition malformed. Expected inclusion of a callback uri in request",
                None,
            )
        })?;

        if let Some(HashMethod::Other(other)) = &finish.hash_method {
            return Err(Errors::not_impl(
                format!("Unsupported hash method {other}"),
                None,
            ));
        }

        let key_source = match &client.key.material {
            KeyMaterial::Jwk { jwk } => DbKeySource::PublicKey(jwk.clone()),
            KeyMaterial::Cert { cert } => DbKeySource::Cert(cert.clone()),
        };

        let host = format!(
            "{}{}/gate",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
        );
        let grant_endpoint = format!("{host}/access");
        let continue_endpoint = format!("{host}/continue");
        let continue_token = create_opaque_token();

        let interaction = interaction::Plan {
            id: id.to_string(),
            start: interact.start,
            method,
            callback_uri,
            key_source,
            client_nonce: finish.nonce,
            hash_method: finish.hash_method,
            hints: interact.hints,
            grant_endpoint,
            continue_endpoint,
            continue_token,
            continue_wait: None,
        };

        Ok(interaction)
    }

    fn build_mate_plan(
        &self,
        holder: &str,
        nick: &str,
        base_url: &str,
        token: &str,
    ) -> participant::Plan {
        let base_url = trim_4_base(&base_url);
        participant::Plan {
            participant_id: holder.to_string(),
            participant_nick: nick.to_string(),
            participant_type: ParticipantType::Agent,
            base_url,
            token: Some(token.to_string()),
            extra_fields: None,
            is_me: false,
        }
    }

    fn validate_grant_req(&self, payload: &Bytes, headers: &HeaderMap) -> Outcome<GrantRequest> {
        info!("Validating grant request");
        let grant_request: GrantRequest = serde_json::from_slice(payload)?;

        match grant_request.client.key.proof {
            KeyProof::HttpSig => {}
            other => {
                return Err(Errors::not_impl(
                    format!("Proof method {} not implemented", other),
                    None,
                ))
            }
        }

        let key_source = match &grant_request.client.key.material {
            KeyMaterial::Jwk { jwk } => {
                let pub_key = PublicKey::parse_from_jwk(jwk)?;
                KeySource::PublicKey(pub_key)
            }
            KeyMaterial::Cert { cert } => {
                let cert = Certificate::try_from_pem(cert)?;
                KeySource::Cert(cert)
            }
        };

        let grant_endpoint = format!(
            "{}{}/gate/access",
            self.config.get_host(HostType::Http),
            self.config.get_api_path()
        );

        HttpSig::verify(headers, &key_source, "POST", &grant_endpoint, payload)?;

        Ok(grant_request)
    }

    fn validate_cont_req(
        &self,
        interaction: &interaction::Model,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()> {
        info!("Validating continuing request");

        let continue_req: ContinueRequest = serde_json::from_slice(payload)?;

        let key_source = match &interaction.key_source {
            DbKeySource::Cert(pem) => {
                let cert = Certificate::try_from_pem(pem)?;
                KeySource::Cert(cert)
            }
            DbKeySource::PublicKey(jwk) => {
                let pub_key = PublicKey::parse_from_jwk(jwk)?;
                KeySource::PublicKey(pub_key)
            }
        };

        HttpSig::verify(
            headers,
            &key_source,
            "POST",
            &interaction.continue_endpoint,
            payload,
        )?;

        if continue_req.interact_ref != interaction.interact_ref {
            return Err(Errors::security(
                &format!(
                    "Interact reference '{}' does not match '{}'",
                    continue_req.interact_ref, interaction.interact_ref,
                ),
                None,
            ));
        }

        let token = extract_gnap_token(headers)?;
        if token != interaction.continue_token {
            return Err(Errors::security(
                &format!(
                    "Token '{}' does not match '{}'",
                    token, interaction.continue_token
                ),
                None,
            ));
        }
        Ok(())
    }

    async fn finish_interaction(
        &self,
        interaction: &interaction::Model,
        verification_result: Outcome<()>,
    ) -> Outcome<InteractionFinishResponse> {
        info!("Finishing interaction");
        match interaction.method {
            FinishMethod::Redirect => match verification_result {
                Ok(_) => {
                    let uri = format!(
                        "{}?hash={}&interact_ref={}",
                        interaction.callback_uri, interaction.hash, interaction.interact_ref
                    );
                    Ok(InteractionFinishResponse::Success(Some(uri)))
                }
                Err(e) => {
                    e.log();
                    let uri = format!("{}?rejected={}", interaction.callback_uri, e);
                    Ok(InteractionFinishResponse::Failure(Some(uri)))
                }
            },
            FinishMethod::Push => {
                let (body, was_approved) = match verification_result {
                    Ok(_) => (
                        CallbackBody::Approved(ApprovedCallbackBody {
                            interact_ref: interaction.interact_ref.clone(),
                            hash: interaction.hash.clone(),
                        }),
                        true,
                    ),
                    Err(e) => {
                        e.log();
                        (
                            CallbackBody::Rejected(RejectedCallbackBody {
                                rejected: e.to_string(),
                            }),
                            false,
                        )
                    }
                };

                http_client()
                    .post(
                        &interaction.callback_uri,
                        Some(json_headers()),
                        HttpBody::json(&body)?,
                    )
                    .await?;

                if was_approved {
                    Ok(InteractionFinishResponse::Success(None))
                } else {
                    Ok(InteractionFinishResponse::Failure(None))
                }
            }
            FinishMethod::Other(_) => {
                unreachable!("build_interaction_plan filters out this state")
            }
        }
    }
}
