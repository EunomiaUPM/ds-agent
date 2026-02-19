/*
 *
 *  * Copyright (C) 2024 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */
use crate::ssi_auth::common::errors::AuthErrors;
use crate::ssi_auth::common::types::entities::{ReachAuthority, InteractStart, WhatEntity};
use crate::ssi_auth::common::types::gnap::{AccessToken, GrantRequest, GrantResponse, RefBody};
use crate::ssi_auth::common::utils::format::trim_4_base;
use crate::ssi_auth::consumer::core::traits::consumer_trait::RainbowSSIAuthConsumerManagerTrait;
use crate::ssi_auth::consumer::core::Manager;
use anyhow::bail;
use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rainbow_common::config::consumer_config::ApplicationConsumerConfigTrait;
use rainbow_common::errors::{helpers::BadFormat, CommonErrors, ErrorLog};
use rainbow_db::auth_consumer::entities::{
    auth_interaction, auth_request, auth_token_requirements, auth_verification, authority_request, mates,
};
use rainbow_db::auth_consumer::repo_factory::factory_trait::AuthRepoFactoryTrait;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::{error, info};
use url::Url;

#[async_trait]
impl<T> RainbowSSIAuthConsumerManagerTrait for Manager<T>
where
    T: AuthRepoFactoryTrait + Send + Sync + Clone + 'static,
{
    async fn request_onboard_provider(
        &self,
        url: String,
        provider_id: String,
        provider_slug: String,
    ) -> anyhow::Result<String> {
        info!("Requesting access to the provider");
        let id = uuid::Uuid::new_v4().to_string();
        let callback_uri = format!(
            "{}/api/v1/callback/{}",
            self.config.get_ssi_auth_host_url().unwrap(),
            &id
        );
        let client = self.config.get_pretty_client_config();
        let mut body = GrantRequest::pr_oidc(client, "push".to_string(), Some(callback_uri));

        let new_request_model =
            auth_request::NewModel { id: id.clone(), provider_id, provider_slug, grant_endpoint: url.clone() };
        let mut request_model = match self.repo.request().create(new_request_model).await {
            Ok(model) => {
                info!("Authentication request saved successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let interact = body.interact.as_ref().unwrap(); // // EXPECTED ALWAYS

        let new_interaction_model = auth_interaction::NewModel {
            id: id.clone(),
            start: interact.start.clone(),
            method: interact.finish.method.clone(),
            uri: interact.finish.uri.as_ref().unwrap().clone(), // // EXPECTED ALWAYS
            hash_method: interact.finish.hash_method.clone(),
            hints: interact.hints.clone(),
            grant_endpoint: url.clone(),
        };

        let mut interaction_model = match self.repo.interaction().create(new_interaction_model).await {
            Ok(model) => {
                info!("Authentication interaction saved successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let new_token_req_model = auth_token_requirements::Model {
            id: id.clone(),
            r#type: "provider-api".to_string(),
            actions: vec!["talk".to_string()],
            locations: None,
            datatypes: None,
            identifier: None,
            privileges: None,
            label: None,
            flags: None,
        };

        let _ = match self.repo.token_requirements().create(new_token_req_model).await {
            Ok(model) => {
                info!("Token requirements saved successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);

        info!("Sending Grant Petition to Provider");

        body.update_nonce(interaction_model.client_nonce.clone());

        let res = self.client.post(&url).headers(headers).json(&body).send().await;

        let res = match res {
            Ok(res) => res,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        let res: GrantResponse = match res.status().as_u16() {
            200 => {
                info!("Grant Response received successfully");
                res.json().await?
            }
            _ => {
                let http_code = Some(res.status().as_u16());
                let error_res: GrantResponse = res.json().await?;
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    http_code,
                    error_res.error,
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let cont_data = match res.r#continue {
            Some(data) => data,
            None => {
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    None,
                    Some("The expected 'continue' field was missing".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        request_model.status = "Pending".to_string();
        request_model.assigned_id = res.instance_id;

        let _ = match self.repo.request().update(request_model).await {
            Ok(model) => {
                info!("Authentication request updated successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let res_interact = match res.interact {
            Some(data) => data,
            None => {
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    None,
                    Some("The expected 'interact' field was missing".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        interaction_model.as_nonce = res_interact.finish;
        interaction_model.continue_token = Some(cont_data.access_token.value);
        interaction_model.continue_endpoint = Some(cont_data.uri);
        interaction_model.continue_wait = cont_data.wait;

        let _ = match self.repo.interaction().update(interaction_model).await {
            Ok(model) => {
                info!("Interaction information updated successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let uri = self.complete_ver_proccess(res_interact.oidc4vp, url, id).await?;

        Ok(uri)
    }

    async fn check_callback(&self, id: String, interact_ref: String, hash: String) -> anyhow::Result<()> {
        info!("Checking callback");
        let mut interaction_model = match self.repo.interaction().get_by_id(&id).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    id.clone(),
                    Some(format!("There is no process with id: {}", &id)),
                );
                error!("{}", error.log());
                bail!(error);
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        interaction_model.interact_ref = Some(interact_ref);
        interaction_model.hash = Some(hash.clone());

        let upd_interaction_model = match self.repo.interaction().update(interaction_model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        // let hash_method = upd_interaction_model.hash_method; // TODO
        let hash_input = format!(
            "{}\n{}\n{}\n{}",
            upd_interaction_model.client_nonce,
            upd_interaction_model.as_nonce.unwrap(),     // EXPECTED ALWAYS
            upd_interaction_model.interact_ref.unwrap(), // EXPECTED ALWAYS
            upd_interaction_model.grant_endpoint
        );

        let mut hasher = Sha256::new(); // TODO
        hasher.update(hash_input.as_bytes());
        let result = hasher.finalize();

        let calculated_hash = URL_SAFE_NO_PAD.encode(result);

        if calculated_hash != hash {
            let error = AuthErrors::security_new(Some("Hash does not match the calculated one".to_string()));
            error!("{}", error.log());
            bail!(error);
        }

        info!("Hash matches the calculated one");
        Ok(())
    }

    async fn continue_request(&self, id: String, interact_ref: String) -> anyhow::Result<Value> {
        // TODO WAIT 5 SECONDS

        let (who, request_model, authority_model) = self.who_is_it(id.clone()).await?;

        let interact_model = match self.repo.interaction().get_by_id(id.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    id.clone(),
                    Some(format!("There is no process with id: {}", &id)),
                );
                error!("{}", error.log());
                bail!(error);
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        match who {
            WhatEntity::Provider => {
                info!("Continuing provider request");

                let url = interact_model.continue_endpoint.unwrap(); // EXPECTED ALWAYS
                let token = format!("GNAP {}", interact_model.continue_token.unwrap()); // EXPECTED ALWAYS

                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, "application/json".parse()?);
                headers.insert(ACCEPT, "application/json".parse()?);
                headers.insert(AUTHORIZATION, token.parse()?);

                let body = RefBody { interact_ref };

                let res = self.client.post(&url).headers(headers).json(&body).send().await;

                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        let http_code = match e.status() {
                            Some(code) => Some(code.as_u16()),
                            None => None,
                        };
                        let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                        error!("{}", error.log());
                        bail!(error);
                    }
                };

                // TODO Is it worth putting "processing" as state??

                let res: AccessToken = match res.status().as_u16() {
                    200 => {
                        info!("Success retrieving the token");
                        res.json().await?
                    }
                    _ => {
                        let http_code = Some(res.status().as_u16());
                        let error_res: GrantResponse = res.json().await?;
                        let error = CommonErrors::provider_new(
                            Some(url),
                            Some("POST".to_string()),
                            http_code,
                            error_res.error,
                        );
                        error!("{}", error.log());
                        bail!(error);
                    }
                };

                let mut request_model = request_model.unwrap(); // EXPECTED ALWAYS
                request_model.status = "Approved".to_string();
                request_model.token = Some(res.value); // TODO Save al token data (pending)

                let upd_request_model = match self.repo.request().update(request_model).await {
                    Ok(model) => model,
                    Err(e) => {
                        let error = CommonErrors::database_new(Some(e.to_string()));
                        error!("{}", error.log());
                        bail!(error);
                    }
                };

                let base_url = trim_4_base(upd_request_model.grant_endpoint.as_str());
                let mate = mates::NewModel {
                    participant_id: upd_request_model.provider_id,
                    participant_slug: upd_request_model.provider_slug,
                    participant_type: "Agent".to_string(),
                    base_url,
                    token: upd_request_model.token,
                    is_me: false,
                };

                let mate = serde_json::to_value(self.save_mate(mate).await?)?;

                Ok(mate)
            }
            WhatEntity::Authority => {
                info!("Continuing authority request");

                let url = interact_model.continue_endpoint.unwrap(); // EXPECTED ALWAYS
                let token = format!("GNAP {}", interact_model.continue_token.unwrap()); // EXPECTED ALWAYS

                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, "application/json".parse()?);
                headers.insert(ACCEPT, "application/json".parse()?);
                headers.insert(AUTHORIZATION, token.parse()?);

                let body = RefBody { interact_ref };

                let res = self.client.post(&url).headers(headers).json(&body).send().await;

                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        let http_code = match e.status() {
                            Some(code) => Some(code.as_u16()),
                            None => None,
                        };
                        let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                        error!("{}", error.log());
                        bail!(error);
                    }
                };

                // TODO Is it worth putting "processing" as state??

                let res = match res.status().as_u16() {
                    200 => {
                        info!("Success retrieving the vc_uri");
                        res.text().await?
                    }
                    _ => {
                        let http_code = Some(res.status().as_u16());
                        let error_res: GrantResponse = res.json().await?;
                        let error = CommonErrors::authority_new(
                            Some(url),
                            Some("POST".to_string()),
                            http_code,
                            error_res.error,
                        );
                        error!("{}", error.log());
                        bail!(error);
                    }
                };

                info!("{}", res);
                let mut authority_model = authority_model.unwrap();
                authority_model.vc_uri = Some(res);
                authority_model.status = "Approved".to_string();

                let upd_request_model = match self.repo.authority().update(authority_model).await {
                    Ok(model) => model,
                    Err(e) => {
                        let error = CommonErrors::database_new(Some(e.to_string()));
                        error!("{}", error.log());
                        bail!(error);
                    }
                };

                let base_url = trim_4_base(upd_request_model.grant_endpoint.as_str());
                let mate = mates::NewModel {
                    participant_id: upd_request_model.authority_id,
                    participant_slug: upd_request_model.authority_slug,
                    participant_type: "Authority".to_string(),
                    base_url,
                    token: None,
                    is_me: false,
                };

                let mate = serde_json::to_value(self.save_mate(mate).await?)?;

                Ok(mate)
            }
        }
    }

    async fn save_mate(&self, mate: mates::NewModel) -> anyhow::Result<mates::Model> {
        match self.repo.mates().force_create(mate).await {
            Ok(model) => Ok(model),
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn complete_ver_proccess(&self, uri: Option<String>, url: String, id: String) -> anyhow::Result<String> {
        info!("Completing verification process saving");

        let uri = match uri {
            Some(data) => data,
            None => {
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    None,
                    Some("The expected 'oidc4vp' field was missing".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let fixed_uri = uri.replacen("openid4vp://", "https://", 1);
        let parsed_uri = Url::parse(&fixed_uri)?;

        let response_type =
            match parsed_uri.query_pairs().find(|(k, _)| k == "response_type").map(|(_, v)| v.into_owned()) {
                Some(data) => data,
                None => {
                    let error = CommonErrors::provider_new(
                        Some(url),
                        Some("POST".to_string()),
                        None,
                        Some("The expected 'response_type' field was missing in the oidc4vp uri".to_string()),
                    );
                    error!("{}", error.log());
                    bail!(error);
                }
            };

        let client_id = match parsed_uri.query_pairs().find(|(k, _)| k == "client_id").map(|(_, v)| v.into_owned()) {
            Some(data) => data,
            None => {
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    None,
                    Some("The expected 'client_id' field was missing in the oidc4vp uri".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let response_mode =
            match parsed_uri.query_pairs().find(|(k, _)| k == "response_mode").map(|(_, v)| v.into_owned()) {
                Some(data) => data,
                None => {
                    let error = CommonErrors::provider_new(
                        Some(url),
                        Some("POST".to_string()),
                        None,
                        Some("The expected 'response_mode' field was missing in the oidc4vp uri".to_string()),
                    );
                    error!("{}", error.log());
                    bail!(error);
                }
            };

        let pd_uri = match parsed_uri
            .query_pairs()
            .find(|(k, _)| k == "presentation_definition_uri")
            .map(|(_, v)| v.into_owned())
        {
            Some(data) => data,
            None => {
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    None,
                    Some("The expected 'presentation_definition_uri' field was missing in the oidc4vp uri".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let client_id_scheme =
            match parsed_uri.query_pairs().find(|(k, _)| k == "client_id_scheme").map(|(_, v)| v.into_owned()) {
                Some(data) => data,
                None => {
                    let error = CommonErrors::provider_new(
                        Some(url),
                        Some("POST".to_string()),
                        None,
                        Some("The expected 'client_id_scheme' field was missing in the oidc4vp uri".to_string()),
                    );
                    error!("{}", error.log());
                    bail!(error);
                }
            };

        let nonce = match parsed_uri.query_pairs().find(|(k, _)| k == "nonce").map(|(_, v)| v.into_owned()) {
            Some(data) => data,
            None => {
                let error = CommonErrors::provider_new(
                    Some(url),
                    Some("POST".to_string()),
                    None,
                    Some("The expected 'nonce' field was missing in the oidc4vp uri".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let response_uri =
            match parsed_uri.query_pairs().find(|(k, _)| k == "response_uri").map(|(_, v)| v.into_owned()) {
                Some(data) => data,
                None => {
                    let error = CommonErrors::provider_new(
                        Some(url),
                        Some("POST".to_string()),
                        None,
                        Some("The expected 'response_uri' field was missing in the oidc4vp uri".to_string()),
                    );
                    error!("{}", error.log());
                    bail!(error);
                }
            };

        let new_verification_model = auth_verification::NewModel {
            id,
            uri,
            scheme: "openid4vp".to_string(),
            response_type,
            client_id,
            response_mode,
            pd_uri,
            client_id_scheme,
            nonce,
            response_uri,
        };

        let verification_model = match self.repo.verification().create(new_verification_model).await {
            Ok(model) => {
                info!("Verification data saved successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        Ok(verification_model.uri.clone())
    }

    async fn beg_credential(&self, payload: ReachAuthority, method: InteractStart) -> anyhow::Result<Option<String>> {
        let authority_id = payload.id;
        let authority_slug = payload.slug;
        let url = payload.url;
        let vc_type = payload.vc_type;

        info!("Begging for a credential");
        let client = self.config.get_pretty_client_config();
        let id = uuid::Uuid::new_v4().to_string();
        let callback_uri = format!(
            "{}/api/v1/callback/{}",
            self.config.get_ssi_auth_host_url().unwrap(),
            &id
        );

        let mut grant_request = match method {
            InteractStart::Oidc => GrantRequest::vc_oidc(
                client,
                "redirect".to_string(),
                Some(callback_uri.clone()),
                vc_type.clone(),
            ),
            InteractStart::CrossUser => GrantRequest::vc_cross_user(client, Some(callback_uri.clone()), vc_type.clone()),
        };

        let new_authority_request_model = authority_request::NewModel {
            id: id.clone(),
            authority_id,
            authority_slug,
            grant_endpoint: url.clone(),
            vc_type,
        };

        let mut authority_model = match self.repo.authority().create(new_authority_request_model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let new_interact_model = auth_interaction::NewModel {
            id: id.clone(),
            start: vec!["await".to_string()],
            method: "push".to_string(),
            uri: callback_uri,
            hash_method: None,
            hints: None,
            grant_endpoint: url.clone(),
        };

        let mut interact_model = match self.repo.interaction().create(new_interact_model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);

        info!("Sending Grant Petition to Authority");

        grant_request.update_nonce(interact_model.client_nonce.clone());

        let res = self.client.post(&url).headers(headers).json(&grant_request).send().await;

        let res = match res {
            Ok(res) => res,
            Err(e) => {
                let http_code = match e.status() {
                    Some(status) => Some(status.as_u16()),
                    None => None,
                };
                let error = CommonErrors::petition_new(url, "POST".to_string(), http_code, e.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };

        let res: GrantResponse = match res.status().as_u16() {
            200 => {
                info!("Grant Response received successfully");
                res.json().await?
            }
            _ => {
                let http_code = Some(res.status().as_u16());
                let error_res: GrantResponse = res.json().await?;
                let error = CommonErrors::authority_new(
                    Some(url),
                    Some("POST".to_string()),
                    http_code,
                    error_res.error,
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        authority_model.status = "Pending".to_string();
        authority_model.assigned_id = res.instance_id;

        let _ = match self.repo.authority().update(authority_model).await {
            Ok(model) => {
                info!("Authority request updated successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let res_interact = match res.interact {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Sent,
                    Some("Missing field interact in the response".to_string()),
                );
                error!("{}", error.log());
                bail!(error)
            }
        };

        let cont_data = match res.r#continue {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Sent,
                    Some("Missing field continue in the response".to_string()),
                );
                error!("{}", error.log());
                bail!(error)
            }
        };

        interact_model.as_nonce = res_interact.finish;
        interact_model.continue_token = Some(cont_data.access_token.value);
        interact_model.continue_endpoint = Some(cont_data.uri);
        interact_model.continue_wait = cont_data.wait;

        let _ = match self.repo.interaction().update(interact_model).await {
            Ok(model) => {
                info!("Authority interaction updated successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        match method {
            InteractStart::CrossUser => Ok(None),
            InteractStart::Oidc => {
                let uri = self.complete_ver_proccess(res_interact.oidc4vp, url, id).await?;
                Ok(Some(uri))
            }
        }
    }

    async fn who_is_it(
        &self,
        id: String,
    ) -> anyhow::Result<(
        WhatEntity,
        Option<auth_request::Model>,
        Option<authority_request::Model>,
    )> {
        // TODO WAIT 5 SECONDS
        info!("Continuing request");

        match self.repo.request().get_by_id(id.as_str()).await {
            Ok(Some(model)) => {
                info!("It is a request 4 the provider");
                return Ok((WhatEntity::Provider, Some(model), None));
            }
            Ok(None) => info!("It is not a request 4 the provider"),
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        match self.repo.authority().get_by_id(id.as_str()).await {
            Ok(Some(model)) => {
                info!("It is a request 4 an authority");
                return Ok((WhatEntity::Authority, None, Some(model)));
            }
            Ok(None) => info!("It is not a request 4 an authority"),
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let error = CommonErrors::missing_resource_new(
            id.clone(),
            Some(format!("Missing resource with id: {}", &id)),
        );
        error!("{}", error.log());
        bail!(error)
    }
}
