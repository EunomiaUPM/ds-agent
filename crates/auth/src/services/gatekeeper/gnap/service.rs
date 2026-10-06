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
use crate::types::token::IssuedToken;
use crate::types::token_lifetimes::{FINAL_TOKEN_TTL_SECS, MANAGING_TOKEN_TTL_SECS};
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use chrono::{DateTime, Duration, Utc};
use common::routes::auth::gate;
use tracing::info;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::received::grant::FinalRotation;
use ymir::data::entities::received::{grant, interaction};
use ymir::data::entities::shared::participant_relation::VERIFICATION_USER_ID;
use ymir::data::entities::shared::{participant, participant_relation, resource_req};
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::http::routes::{base, fill};
use ymir::services::client::ClientTrait;
use ymir::types::gnap::access_token::{BoundToken, TokenManagement};
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
use ymir::types::oauth::RolePath;
use ymir::types::participants::{ParticipantType, Visibility};
use ymir::utils::{
    create_opaque_token, extract_gnap_token, hash_token, http_client, json_headers, trim_4_base,
};

/// GNAP gatekeeper.
pub struct GnapGateKeeperService {
    config: GnapGateKeeperConfig,
}

impl GnapGateKeeperService {
    pub fn new(config: GnapGateKeeperConfig) -> GnapGateKeeperService {
        GnapGateKeeperService { config }
    }

    fn key_source(interaction: &interaction::Model) -> Outcome<KeySource> {
        Ok(match &interaction.key_source {
            DbKeySource::Cert(pem) => KeySource::Cert(Certificate::try_from_pem(pem)?),
            DbKeySource::PublicKey(jwk) => KeySource::PublicKey(PublicKey::parse_from_jwk(jwk)?),
        })
    }

    fn managing_uri(&self, managing_id: &str) -> String {
        format!(
            "{}{}{}{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
            gate::PREFIX,
            fill(gate::TOKEN, managing_id),
        )
    }

    fn seconds_until(at: DateTime<Utc>, now: DateTime<Utc>) -> u64 {
        (at - now).num_seconds().max(0) as u64
    }

    fn new_tokens(
        &self,
        managing_id: &str,
        managing_expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> (FinalRotation, IssuedToken) {
        let final_token = create_opaque_token();
        let managing_token = create_opaque_token();
        let final_expires_at =
            (now + Duration::seconds(FINAL_TOKEN_TTL_SECS)).min(managing_expires_at);

        let rotation = FinalRotation {
            final_token_hash: hash_token(&final_token),
            final_expires_at,
            managing_token_hash: hash_token(&managing_token),
        };
        let managing = BoundToken::new(managing_token)
            .with_expires_in(Self::seconds_until(managing_expires_at, now));
        let issued = IssuedToken {
            final_token,
            final_expires_in: Self::seconds_until(final_expires_at, now),
            manage: TokenManagement::new(self.managing_uri(managing_id), managing),
        };
        (rotation, issued)
    }
}

#[async_trait]
impl GateKeeperTrait for GnapGateKeeperService {
    fn build_grant_plan(
        &self,
        role: Option<RolePath>,
        visibility: Option<Visibility>,
        class_id: Option<String>,
    ) -> Outcome<grant::Plan> {
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
            role: role.unwrap_or_else(RolePath::root),
            visibility: visibility.unwrap_or(Visibility::Public),
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
            "{}{}{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
            gate::PREFIX,
        );
        let grant_endpoint = format!("{host}{}", gate::ACCESS);
        let continuation_endpoint = format!("{host}{}", base(gate::CONTINUE));
        let continuation_token = create_opaque_token();

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
            continuation_endpoint,
            continuation_token,
            continuation_wait: None,
        };

        Ok(interaction)
    }

    fn build_mate_plan(&self, holder: &str, nick: &str, base_url: &str) -> participant::Plan {
        let base_url = trim_4_base(&base_url);
        participant::Plan {
            participant_id: holder.to_string(),
            participant_nick: nick.to_string(),
            participant_type: ParticipantType::Agent,
            base_url,
            extra_fields: None,
        }
    }

    fn build_mate_rel_plan(&self, role: &RolePath, holder: &str) -> participant_relation::Model {
        participant_relation::Model {
            user_id: VERIFICATION_USER_ID.to_string(),
            username: None,
            participant_id: holder.to_string(),
            role: role.clone(),
            visibility: Visibility::Private,
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
            "{}{}{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_path(),
            gate::PREFIX,
            gate::ACCESS,
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

        let key_source = Self::key_source(interaction)?;

        HttpSig::verify(
            headers,
            &key_source,
            "POST",
            &interaction.continuation_endpoint,
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
        if token != interaction.continuation_token {
            return Err(Errors::security(
                &format!(
                    "Token '{}' does not match '{}'",
                    token, interaction.continuation_token
                ),
                None,
            ));
        }
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err)]
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

    fn issue_token(&self, grant: &mut grant::Model, now: DateTime<Utc>) -> IssuedToken {
        let managing_id = create_opaque_token();
        let managing_expires_at = now + Duration::seconds(MANAGING_TOKEN_TTL_SECS);
        let (rotation, issued) = self.new_tokens(&managing_id, managing_expires_at, now);
        grant.managing_id = Some(managing_id);
        grant.final_token_hash = Some(rotation.final_token_hash);
        grant.final_expires_at = Some(rotation.final_expires_at);
        grant.managing_token_hash = Some(rotation.managing_token_hash);
        grant.managing_expires_at = Some(managing_expires_at);
        issued
    }

    fn rotate_token(
        &self,
        grant: &grant::Model,
        now: DateTime<Utc>,
    ) -> Outcome<(FinalRotation, IssuedToken)> {
        let managing_expires_at = match grant.managing_expires_at {
            Some(at) if at > now => at,
            _ => {
                return Err(Errors::security(
                    "Grant can no longer rotate its token",
                    None,
                ))
            }
        };
        let managing_id = grant
            .managing_id
            .as_deref()
            .ok_or_else(|| Errors::security("Grant has no token management", None))?;
        Ok(self.new_tokens(managing_id, managing_expires_at, now))
    }

    fn validate_managing_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
        method: &str,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()> {
        info!("Validating token management request");

        let managing_id = grant
            .managing_id
            .as_deref()
            .ok_or_else(|| Errors::security("Grant has no token management", None))?;
        let key_source = Self::key_source(interaction)?;
        HttpSig::verify(
            headers,
            &key_source,
            method,
            &self.managing_uri(managing_id),
            payload,
        )?;

        let token = extract_gnap_token(headers)?;
        if grant.managing_token_hash.as_deref() != Some(hash_token(&token).as_str()) {
            return Err(Errors::security("Managing token does not match", None));
        }
        Ok(())
    }
}
