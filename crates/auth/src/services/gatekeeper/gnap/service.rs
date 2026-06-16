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

use axum::body::Bytes;
use axum::http::HeaderMap;
use tracing::info;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::received::{grant, interaction, verification};
use ymir::data::entities::shared::{participant, resource_req};
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::services::client::ClientTrait;
use ymir::types::gnap::grant_request::access::AccessRequest;
use ymir::types::gnap::grant_request::client::{Client, KeyMaterial, KeyProof};
use ymir::types::gnap::grant_request::interact::{
    FinishMethod, HashMethod, InteractAction, InteractRequest, InteractStart,
};
use ymir::types::gnap::grant_request::{GrantKind, GrantRequest, GrantRequestKind};
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{ApprovedCallbackBody, ContinueRequest, GrantStatus};
use ymir::types::http::HttpBody;
use ymir::types::keys::Certificate;
use ymir::types::participants::ParticipantType;
use ymir::utils::{
    create_opaque_token, extract_gnap_token, http_client, json_headers, trim_4_base,
};

use super::super::GateKeeperTrait;
use super::config::GnapGateKeeperConfig;

pub struct GnapGateKeeperService {
    config: GnapGateKeeperConfig,
}

impl GnapGateKeeperService {
    pub fn new(config: GnapGateKeeperConfig) -> GnapGateKeeperService {
        GnapGateKeeperService { config }
    }
}

impl GateKeeperTrait for GnapGateKeeperService {
    fn validate_grant(&self, payload: &Bytes, headers: &HeaderMap) -> Outcome<GrantRequest> {
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

        let cert = match &grant_request.client.key.material {
            KeyMaterial::Jwk { .. } => {
                return Err(Errors::not_impl("jwk key material not implemented", None))
            }
            KeyMaterial::Cert { cert } => Certificate::try_from_pem(cert)?,
        };

        let grant_endpoint = format!(
            "{}{}/gate/access",
            self.config.get_host(HostType::Http),
            self.config.get_api_path()
        );

        HttpSig::verify(headers, &cert, "POST", &grant_endpoint, payload)?;

        Ok(grant_request)
    }

    fn build_grant_plan(
        &self,
        payload: GrantRequest,
    ) -> Outcome<(grant::Plan, interaction::Plan, resource_req::Model)> {
        info!("Managing Grant Request");

        let v = Self::validate_and_extract(payload)?;

        let id = uuid::Uuid::new_v4().to_string();
        let host = format!(
            "{}{}/gate",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
        );
        let grant_endpoint = format!("{host}/access");
        let continue_endpoint = format!("{host}/continue");
        let continue_token = create_opaque_token();

        let grant = grant::Plan {
            id: id.clone(),
            participant_nick: v.class_id,
            kind: GrantKind::AccessToken,
        };

        let interaction = interaction::Plan {
            id: id.clone(),
            start: v.interact_start,
            method: v.method,
            callback_uri: v.callback_uri,
            cert: v.cert,
            client_nonce: v.client_nonce,
            hash_method: v.hash_method,
            hints: v.hints,
            grant_endpoint,
            continue_endpoint,
            continue_token,
            continue_wait: None,
        };

        let resource_req = resource_req::Model {
            id,
            r#type: v.access_req.access.r#type,
            actions: v.actions,
            locations: v.access_req.access.locations,
            datatypes: v.access_req.access.datatypes,
            identifier: v.access_req.access.identifier,
            privileges: v.access_req.access.privileges,
            label: v.access_req.label,
            flags: v.access_req.flags,
        };

        Ok((grant, interaction, resource_req))
    }

    fn respond_grant_pending(&self, int_model: &interaction::Model, uri: &str) -> GrantResponse {
        info!("Generating Grant Response");
        GrantResponse::pending(uri, &int_model)
    }

    fn validate_cont_req(
        &self,
        model: &interaction::Model,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()> {
        info!("Validating continuing request");

        let continue_req: ContinueRequest = serde_json::from_slice(payload)?;

        let token = extract_gnap_token(headers)?;

        let cert = Certificate::try_from_pem(&model.cert)?;

        HttpSig::verify(headers, &cert, "POST", &model.continue_endpoint, payload)?;

        if continue_req.interact_ref != model.interact_ref {
            return Err(Errors::security(
                &format!(
                    "Interact reference '{}' does not match '{}'",
                    continue_req.interact_ref, model.interact_ref,
                ),
                None,
            ));
        }

        if token != model.continue_token {
            return Err(Errors::security(
                &format!(
                    "Token '{}' does not match '{}'",
                    token, model.continue_token
                ),
                None,
            ));
        }
        Ok(())
    }

    async fn finish_interaction(&self, model: &interaction::Model) -> Outcome<Option<String>> {
        info!("Finishing interaction");
        match model.method {
            FinishMethod::Redirect => {
                let uri = format!(
                    "{}?hash={}&interact_ref={}",
                    model.callback_uri, model.hash, model.interact_ref
                );
                Ok(Some(uri))
            }
            FinishMethod::Push => {
                let body = ApprovedCallbackBody {
                    interact_ref: model.interact_ref.clone(),
                    hash: model.hash.clone(),
                };
                http_client()
                    .post(
                        &model.callback_uri,
                        Some(json_headers()),
                        HttpBody::json(&body)?,
                    )
                    .await?;
                Ok(None)
            }
            FinishMethod::Other(_) => {
                unreachable!("build_grant_plan filters out this state")
            }
        }
    }

    fn end_req(
        &self,
        grant: &mut grant::Model,
        resource_req: &resource_req::Model,
        token: &str,
    ) -> GrantResponse {
        info!("Continuing Request");

        grant.token = Some(token.to_string());
        grant.status = GrantStatus::Approved;

        GrantResponse::token_approved(token, resource_req)
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
}

struct ValidatedGrant {
    cert: String,
    class_id: String,
    interact_start: Vec<InteractStart>,
    hints: Option<String>,
    method: FinishMethod,
    callback_uri: String,
    client_nonce: String,
    hash_method: Option<HashMethod>,
    actions: Vec<InteractAction>,
    access_req: AccessRequest,
}

impl GnapGateKeeperService {
    fn validate_and_extract(payload: GrantRequest) -> Outcome<ValidatedGrant> {
        let interact = payload.interact.ok_or_else(|| {
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

        let cert = match payload.client.key.material {
            KeyMaterial::Jwk { .. } => {
                return Err(Errors::not_impl("jwk key material not implemented", None))
            }
            KeyMaterial::Cert { cert } => cert,
        };

        let class_id = payload.client.class_id.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Missing field class_id (used for nick) in the petition",
                None,
            )
        })?;

        let access_req = match payload.kind {
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

        let finish = interact.finish.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Expected inclusion of finish indicator in request",
                None,
            )
        })?;
        let callback_uri = finish.uri.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Expected inclusion of a callback uri in request",
                None,
            )
        })?;

        if let Some(HashMethod::Other(other)) = &finish.hash_method {
            return Err(Errors::not_impl(
                format!("Unsupported hash method {other}"),
                None,
            ));
        }

        let method = match finish.method {
            FinishMethod::Other(other) => {
                return Err(Errors::not_impl(
                    format!("Interact method {other} not supported"),
                    None,
                ))
            }
            supported => supported,
        };

        Ok(ValidatedGrant {
            cert,
            class_id,
            interact_start: interact.start,
            hints: interact.hints,
            method,
            callback_uri,
            client_nonce: finish.nonce,
            hash_method: finish.hash_method,
            actions,
            access_req,
        })
    }
}
