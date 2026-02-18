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

use std::sync::Arc;

use anyhow::bail;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use reqwest::Response;
use tracing::{error, info};
use url::Url;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::{
    mates, req_interaction, req_request, req_verification, token_requirements,
};
use ymir::errors::{ErrorLogTrait, Errors};
use ymir::services::client::ClientTrait;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::gnap::grant_request::{GrantRequest, InteractStart};
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{AccessToken, GRUse};
use ymir::types::http::Body;
use ymir::types::secrets::StringHelper;
use ymir::utils::{expect_from_env, get_from_opt, get_query_param, trim_4_base};

use super::super::OnboarderTrait;
use crate::ssi::services::onboarder::gnap::config::{
    GnapOnboarderConfig, GnapOnboarderConfigTrait,
};
use crate::ssi::types::entities::ReachProvider;

pub struct GnapOnboarderService {
    client: Arc<dyn ClientTrait>,
    vault: Arc<VaultService>,
    config: GnapOnboarderConfig,
}

impl GnapOnboarderService {
    pub fn new(
        client: Arc<dyn ClientTrait>,
        vault: Arc<VaultService>,
        config: GnapOnboarderConfig,
    ) -> GnapOnboarderService {
        GnapOnboarderService { client, vault, config }
    }
}

#[async_trait]
impl OnboarderTrait for GnapOnboarderService {
    fn start(
        &self,
        payload: &ReachProvider,
    ) -> (
        req_request::NewModel,
        req_interaction::NewModel,
        token_requirements::Model,
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
        };

        let int_model = req_interaction::NewModel {
            id: id.clone(),
            start: vec![InteractStart::Oidc4VP.to_string()],
            method: "push".to_string(),
            uri: callback_uri.clone(),
            hash_method: Some("sha-256".to_string()),
            hints: None,
            grant_endpoint: payload.url.clone(),
        };

        let token_model = token_requirements::Model {
            id,
            r#type: "provider-api".to_string(),
            actions: vec!["talk".to_string()],
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None,
        };

        (req_model, int_model, token_model)
    }

    async fn send_req(
        &self,
        req_model: &mut req_request::Model,
        int_model: &mut req_interaction::Model,
    ) -> anyhow::Result<()> {
        info!("Sending onboarding request");

        let cert = expect_from_env("VAULT_APP_CERT");
        let cert: StringHelper = self.vault.read(None, &cert).await?;
        let client = self.config.get_pretty_client_config(&cert.data())?;

        let grant_request = GrantRequest::new(GRUse::Talk, client, None, &int_model);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);

        let res = self
            .client
            .post(
                &int_model.grant_endpoint,
                Some(headers),
                Body::Json(serde_json::to_value(grant_request)?),
            )
            .await?;

        let res: GrantResponse = match res.status().as_u16() {
            200 => {
                info!("Grant Response received successfully");
                res.json().await?
            }
            _ => {
                let http_code = Some(res.status().as_u16());
                let error_res: GrantResponse = res.json().await?;
                let error = Errors::provider_new(
                    &int_model.grant_endpoint,
                    "POST",
                    http_code,
                    &error_res.error.unwrap(),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        req_model.status = "Pending".to_string();
        req_model.assigned_id = res.instance_id;

        let interact = get_from_opt(&res.interact, "interact")?;
        let cont_data = get_from_opt(&res.r#continue, "continue")?;

        int_model.as_nonce = interact.finish;
        int_model.oidc_vp_uri = interact.oidc4vp;
        int_model.continue_token = Some(cont_data.access_token.value);
        int_model.continue_endpoint = Some(cont_data.uri);
        int_model.continue_wait = cont_data.wait;
        Ok(())
    }

    fn save_verification(
        &self,
        int_model: &req_interaction::Model,
    ) -> anyhow::Result<req_verification::NewModel> {
        info!("Saving verification data");

        let uri = get_from_opt(&int_model.oidc_vp_uri, "oidc4vp")?;
        let fixed_uri = uri.replacen("openid4vp://", "https://", 1);
        let parsed_uri = Url::parse(&fixed_uri)?;

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
            response_uri,
        })
    }

    async fn manage_res(
        &self,
        req_model: &mut req_request::Model,
        res: Response,
    ) -> anyhow::Result<mates::NewModel> {
        info!("Managing response");
        let token = match res.status().as_u16() {
            200 => {
                info!("Success retrieving the token");
                let token: AccessToken = res.json().await?;
                token
            }
            _ => {
                let http_code = Some(res.status().as_u16());
                let error_res: GrantResponse = res.json().await?;
                let error = Errors::provider_new(
                    "provider/continue",
                    "POST",
                    http_code,
                    &error_res.error.unwrap_or("Error with authority continue request".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        req_model.status = "Approved".to_string();
        req_model.token = Some(token.value.clone());

        let base_url = trim_4_base(&req_model.grant_endpoint);
        let mates = mates::NewModel {
            participant_id: req_model.provider_id.clone(),
            participant_slug: req_model.provider_slug.clone(),
            participant_type: "Provider".to_string(),
            base_url,
            token: req_model.token.clone(),
            is_me: false,
        };
        Ok(mates)
    }
    async fn manage_rejection(&self, model: &mut req_request::Model) -> anyhow::Result<()> {
        model.status = "Rejected".to_string();
        model.ended_at = Some(chrono::Utc::now().naive_utc());
        Ok(())
    }
}
