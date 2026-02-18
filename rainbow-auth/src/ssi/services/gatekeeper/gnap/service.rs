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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::bail;
use tracing::{error, info};
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::{
    mates, recv_interaction, recv_request, recv_verification, token_requirements,
};
use ymir::errors::{ErrorLogTrait, Errors};
use ymir::types::errors::BadFormat;
use ymir::types::gnap::grant_request::{GrantRequest, InteractStart};
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{AccessToken, RefBody};
use ymir::utils::get_from_opt;
use ymir::utils::{create_opaque_token, trim_4_base};

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
        payload: &GrantRequest,
    ) -> anyhow::Result<(
        recv_request::NewModel,
        recv_interaction::NewModel,
        token_requirements::Model,
    )> {
        info!("Managing Grant Request");

        let interact = get_from_opt(&payload.interact, "interact")?;

        if !&interact.start.contains(&"oidc4vp".to_string()) {
            let error = Errors::not_impl_new(
                "Interact method not supported yet",
                "Interact method not supported yet",
            );
            error!("{}", error.log());
            bail!(error);
        }

        let class_id = payload.client.class_id.as_ref().ok_or_else(|| {
            let error =
                Errors::format_new(BadFormat::Received, "Missing field class_id in the petition");
            error!("{}", error.log());
            error
        })?;

        let uri = get_from_opt(&interact.finish.uri, "interact finish uri")?;
        let id = uuid::Uuid::new_v4().to_string();

        let req_model = recv_request::NewModel { id: id.clone(), consumer_slug: class_id.clone() };

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
            client_nonce: interact.finish.nonce,
            hash_method: interact.finish.hash_method,
            hints: interact.hints,
            grant_endpoint,
            continue_endpoint,
            continue_token,
        };

        let token_model = token_requirements::Model {
            id: id.clone(),
            r#type: payload.access_token.access.r#type.clone(),
            actions: payload
                .access_token
                .access
                .actions
                .clone()
                .unwrap_or(vec![String::from("talk")]),
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None,
        };

        Ok((req_model, int_model, token_model))
    }

    fn respond_req(&self, int_model: &recv_interaction::Model, uri: &str) -> GrantResponse {
        info!("Generating Grant Response");
        GrantResponse::new(InteractStart::Oidc4VP, int_model, Some(uri.to_string()))
    }

    fn validate_cont_req(
        &self,
        model: &recv_interaction::Model,
        payload: &RefBody,
        token: &str,
    ) -> anyhow::Result<()> {
        info!("Validating continuing request");

        if payload.interact_ref.clone() != model.interact_ref.clone() {
            let error = Errors::security_new(&format!(
                "Interact reference '{}' does not match '{}'",
                payload.interact_ref, model.interact_ref
            ));
            error!("{}", error.log());
            bail!(error);
        }

        if token != model.continue_token {
            let error = Errors::security_new(&format!(
                "Token '{}' does not match '{}'",
                token, model.continue_token
            ));
            error!("{}", error.log());
            bail!(error);
        }
        Ok(())
    }

    fn continue_req(
        &self,
        req_model: &mut recv_request::Model,
        int_model: &recv_interaction::Model,
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
            participant_type: "Consumer".to_string(),
            base_url,
            token: Some(token.clone()),
            is_me: false,
        };

        let token = AccessToken::default(token);
        (mate, token)
    }
}
