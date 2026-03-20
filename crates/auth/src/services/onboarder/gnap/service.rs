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

use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use common::config::types::traits::EntityClientTrait;
use reqwest::Response;
use tracing::info;
use ymir::capabilities::HttpSig;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::{
    mates, req_interaction, req_request, req_verification, token_requirements
};
use ymir::errors::{Errors, Outcome};
use ymir::services::client::ClientTrait;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::gnap::grant_request::{GrantRequest, InteractActions, InteractStart};
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::AccessToken;
use ymir::types::http::Body;
use ymir::types::secrets::StringHelper;
use ymir::utils::{
    expect_from_env, get_from_opt, get_query_param, json_headers, trim_4_base, ResponseExt
};

use super::super::OnboarderTrait;
use crate::services::onboarder::gnap::config::{
    GnapOnboarderConfig, GnapOnboarderConfigTrait
};
use crate::types::entities::ReachProvider;
use crate::utils::parse_url;

pub struct GnapOnboarderService {
    client: Arc<dyn ClientTrait>,
    vault: Arc<VaultService>,
    config: GnapOnboarderConfig
}

impl GnapOnboarderService {
    pub fn new(
        client: Arc<dyn ClientTrait>,
        vault: Arc<VaultService>,
        config: GnapOnboarderConfig
    ) -> GnapOnboarderService {
        GnapOnboarderService { client, vault, config }
    }
}

#[async_trait]
impl OnboarderTrait for GnapOnboarderService {
    fn start(
        &self,
        payload: &ReachProvider
    ) -> (
        req_request::NewModel,
        req_interaction::NewModel,
        token_requirements::Model
    ) {
        info!("Starting process to request consumer onboarding");

        let id = uuid::Uuid::new_v4().to_string();
        let callback_uri = format!(
            "{}{}/onboard/callback/{}",
            self.config.hosts().get_host(HostType::Http),
            self.config.get_api_path(),
            &id
        );

        let req_model = req_request::NewModel {
            id: id.clone(),
            provider_id: payload.id.clone(),
            provider_slug: payload.slug.clone(),
            grant_endpoint: payload.url.clone(),
            auto: payload.auto
        };

        let int_model = req_interaction::NewModel {
            id: id.clone(),
            start: vec![InteractStart::Oidc4VP.to_string()],
            method: "push".to_string(),
            uri: callback_uri.clone(),
            hash_method: Some("sha-256".to_string()),
            hints: None,
            grant_endpoint: payload.url.clone()
        };

        // TO VALIDATE ACTIONS
        let actions: Vec<InteractActions> = payload
            .actions
            .iter()
            .filter_map(|data| InteractActions::from_str(data).ok())
            .collect();

        let actions = if actions.is_empty() {
            vec![InteractActions::Talk.to_string()]
        } else {
            actions.iter().map(|data| data.to_string()).collect()
        };

        let token_model = token_requirements::Model {
            id,
            r#type: "provider-api".to_string(),
            actions,
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None
        };

        (req_model, int_model, token_model)
    }

    async fn send_req(
        &self,
        req_model: &mut req_request::Model,
        int_model: &mut req_interaction::Model
    ) -> Outcome<()> {
        info!("Sending onboarding request");

        let cert = expect_from_env("VAULT_APP_CERT");
        let cert: StringHelper = self.vault.read(None, &cert).await?;
        let key = expect_from_env("VAULT_APP_PRIV_KEY");
        let key: StringHelper = self.vault.read(None, &key).await?;

        let client = self.config.get_pretty_client_config(cert.data())?;
        let grant_request = GrantRequest::new_token(&client, None, &int_model);

        let (body, body_bytes) = Body::from_json_bytes(&grant_request)?;

        let mut headers = json_headers();
        let httpsig = HttpSig::build(
            cert.data(),
            key.data(),
            "POST",
            &req_model.grant_endpoint,
            &body_bytes,
            None
        )?;

        headers.extend(httpsig);

        let res = self.client.post(&int_model.grant_endpoint, Some(headers), body).await?;

        let res: GrantResponse = if res.status().is_success() {
            info!("Grant Response received successfully");
            res.parse_json().await?
        } else {
            let status = res.status();
            let error_res: GrantResponse = res.parse_json().await?;
            return Err(Errors::provider(
                &int_model.grant_endpoint,
                "POST",
                Some(status),
                error_res.error.unwrap_or("Unexpected error on provider onboarding".to_string()),
                None
            ));
        };

        req_model.status = "Pending".to_string();
        req_model.assigned_id = res.instance_id;

        let interact = get_from_opt(res.interact.as_ref(), "interact")?;
        let cont_data = get_from_opt(res.r#continue.as_ref(), "continue")?;

        int_model.as_nonce = interact.finish;
        int_model.oidc_vp_uri = interact.oidc4vp;
        int_model.continue_token = Some(cont_data.access_token.value);
        int_model.continue_endpoint = Some(cont_data.uri);
        int_model.continue_wait = cont_data.wait;
        Ok(())
    }

    fn save_verification(
        &self,
        int_model: &req_interaction::Model
    ) -> Outcome<req_verification::NewModel> {
        info!("Saving verification data");

        let uri = get_from_opt(int_model.oidc_vp_uri.as_ref(), "oidc4vp")?;
        let fixed_uri = uri.replacen("openid4vp://", "https://", 1);
        let parsed_uri = parse_url(&fixed_uri)?;

        let response_type = get_query_param(&parsed_uri, "response_type")?;
        let client_id = get_query_param(&parsed_uri, "client_id")?;
        let response_mode = get_query_param(&parsed_uri, "response_mode")?;
        let pd_uri = get_query_param(&parsed_uri, "presentation_definition_uri")?;
        let client_id_scheme = get_query_param(&parsed_uri, "client_id_scheme")?;
        let nonce = get_query_param(&parsed_uri, "nonce")?;
        let response_uri = get_query_param(&parsed_uri, "response_uri")?;

        Ok(req_verification::NewModel {
            id: int_model.id.clone(),
            uri,
            scheme: "openid4vp".to_string(),
            response_type,
            client_id,
            response_mode,
            pd_uri,
            client_id_scheme,
            nonce,
            response_uri
        })
    }

    async fn manage_res(
        &self,
        req_model: &mut req_request::Model,
        res: Response
    ) -> Outcome<mates::NewModel> {
        info!("Managing response");
        let token = if res.status().is_success() {
            info!("Success retrieving the token");
            let token: AccessToken = res.parse_json().await?;
            token
        } else {
            return Err(Errors::provider(
                res.url().to_string(),
                "POST",
                Some(res.status()),
                "Error with provider continue request",
                None
            ));
        };

        req_model.status = "Approved".to_string();
        req_model.token = Some(token.value);

        let base_url = trim_4_base(&req_model.grant_endpoint);
        let mates = mates::NewModel {
            participant_id: req_model.provider_id.clone(),
            participant_slug: req_model.provider_slug.clone(),
            participant_type: "Agent".to_string(),
            base_url,
            token: req_model.token.clone(),
            is_me: false
        };
        Ok(mates)
    }
    async fn manage_rejection(&self, model: &mut req_request::Model) -> Outcome<()> {
        model.status = "Rejected".to_string();
        model.ended_at = Some(chrono::Utc::now().naive_utc());
        Ok(())
    }
}
