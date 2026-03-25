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
use ymir::data::entities::{
    mates, recv_interaction, recv_request, recv_verification, token_requirements,
};
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::gnap::grant_request::{GrantRequest, InteractActions, InteractStart, KeyProof};
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{AccessToken, RefBody};
use ymir::utils::{create_opaque_token, extract_gnap_token, trim_4_base};
use ymir::utils::{get_from_opt, parse_from_slice};

use super::super::GateKeeperTrait;
use super::config::{GnapGateKeeperConfig, GnapGateKeeperConfigTrait};

pub struct GnapGateKeeperService {
    config: GnapGateKeeperConfig,
}

impl GnapGateKeeperService {
    pub fn new(config: GnapGateKeeperConfig) -> GnapGateKeeperService {
        GnapGateKeeperService { config }
    }
}

impl GateKeeperTrait for GnapGateKeeperService {
    fn start(
        &self,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<(
        recv_request::NewModel,
        recv_interaction::NewModel,
        token_requirements::Model,
    )> {
        info!("Managing Grant Request");

        let payload = self.validate_req(payload, headers)?;

        let interact = get_from_opt(payload.interact.as_ref(), "interact")?;

        if !&interact.start.contains(&"oidc4vp".to_string()) {
            return Err(Errors::not_impl("Interact method not supported yet", None));
        }

        let cert = payload.client.key.cert.ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Right now only petitions including a cert are accepted",
                None,
            )
        })?;
        let class_id = payload.client.class_id.as_ref().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Missing field class_id in the petition",
                None,
            )
        })?;

        let tok_req = payload.access_token.as_ref().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Missing field access_token in the Grant Request",
                None,
            )
        })?;

        let actions: Vec<InteractActions> = tok_req
            .access
            .actions
            .as_deref()
            .unwrap_or(&["talk".to_string()])
            .iter()
            .filter_map(|data| InteractActions::from_str(data).ok())
            .collect();

        let actions = if actions.is_empty() {
            vec![InteractActions::Talk.to_string()]
        } else {
            actions.iter().map(|data| data.to_string()).collect()
        };

        let uri = get_from_opt(interact.finish.uri.as_ref(), "interact finish uri")?;
        let id = uuid::Uuid::new_v4().to_string();

        let req_model = recv_request::NewModel {
            id: id.clone(),
            consumer_slug: class_id.clone(),
        };

        let host = format!(
            "{}{}/gate",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path()
        );

        let grant_endpoint = format!("{}/access", &host);
        let continue_endpoint = format!("{}/continue", &host);
        let continue_token = create_opaque_token();

        let int_model = recv_interaction::NewModel {
            id: id.clone(),
            start: interact.start,
            method: interact.finish.method,
            uri,
            cert,
            client_nonce: interact.finish.nonce,
            hash_method: interact.finish.hash_method,
            hints: interact.hints,
            grant_endpoint,
            continue_endpoint,
            continue_token,
        };

        let token_model = token_requirements::Model {
            id,
            r#type: tok_req.access.r#type.clone(),
            actions,
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None,
        };

        Ok((req_model, int_model, token_model))
    }

    fn validate_req(&self, payload: &Bytes, headers: &HeaderMap) -> Outcome<GrantRequest> {
        let grant_request: GrantRequest = parse_from_slice(payload)?;

        match grant_request.client.key.cert.as_deref() {
            Some(cert) => {
                let proof = KeyProof::from_str(&grant_request.client.key.proof)?;
                match proof {
                    KeyProof::HttpSig => {}
                    method => {
                        return Err(Errors::not_impl(
                            format!("Right now we only accept httpsig, not {}", method),
                            None,
                        ))
                    }
                }

                let grant_endpoint = format!(
                    "{}{}/gate/access",
                    self.config.get_host(HostType::Http),
                    self.config.get_api_path()
                );

                HttpSig::verify(headers, "POST", &grant_endpoint, payload, &cert)?;

                HttpSig::check_cert(&cert)?;
            }
            None => {
                if let Some(_) = grant_request.client.key.jwk.as_ref() {
                    return Err(Errors::not_impl(
                        "Cannot make this flow with jwk yet, try with cert",
                        None,
                    ));
                }
                return Err(Errors::format(
                    BadFormat::Received,
                    "Client certificate has not arrived",
                    None,
                ));
            }
        }

        Ok(grant_request)
    }

    fn respond_req(&self, int_model: &recv_interaction::Model, uri: &str) -> GrantResponse {
        info!("Generating Grant Response");
        GrantResponse::pending(&InteractStart::Oidc4VP, int_model, Some(uri))
    }

    fn validate_cont_req(
        &self,
        model: &recv_interaction::Model,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()> {
        info!("Validating continuing request");

        let ref_body: RefBody = parse_from_slice(payload)?;

        let token = extract_gnap_token(headers)?;

        HttpSig::verify(
            headers,
            "POST",
            &model.continue_endpoint,
            payload,
            &model.cert,
        )?;

        HttpSig::check_cert(&model.cert)?;

        if ref_body.interact_ref.clone() != model.interact_ref.clone() {
            return Err(Errors::security(
                &format!(
                    "Interact reference '{}' does not match '{}'",
                    ref_body.interact_ref, model.interact_ref,
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

    fn continue_req(
        &self,
        req_model: &mut recv_request::Model,
        int_model: &recv_interaction::Model,
        token_model: &token_requirements::Model,
        ver_model: &recv_verification::Model,
    ) -> (mates::NewModel, AccessToken) {
        info!("Continuing Request");

        let token = create_opaque_token();
        req_model.token = Some(token.clone());
        req_model.status = "Approved".to_string();

        let base_url = trim_4_base(&int_model.uri);
        let mate = mates::NewModel {
            participant_id: ver_model.holder.clone().unwrap(),
            participant_slug: req_model.consumer_slug.clone(),
            participant_type: "Agent".to_string(),
            base_url,
            token: Some(token.clone()),
            is_me: false,
        };

        let token = AccessToken::new(token, token_model);
        (mate, token)
    }
}
