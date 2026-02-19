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
use crate::ssi_auth::common::types::gnap::{CallbackBody, GrantRequest, GrantResponse};
use crate::ssi_auth::common::utils::format::{split_did, trim_4_base};
use crate::ssi_auth::common::utils::token::create_opaque_token;
use crate::ssi_auth::provider::core::traits::provider_trait::RainbowSSIAuthProviderManagerTrait;
use crate::ssi_auth::provider::core::Manager;
use anyhow::bail;
use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::Validation;
use rainbow_common::config::provider_config::ApplicationProviderConfigTrait;
use rainbow_common::errors::helpers::{BadFormat, MissingAction};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use rainbow_db::auth_provider::entities::{
    auth_interaction, auth_request, auth_token_requirements, auth_verification, business_mates, mates,
};
use rainbow_db::auth_provider::repo_factory::factory_trait::AuthRepoFactoryTrait;
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use serde_json::{json, Value};
use std::collections::HashSet;
use tracing::{debug, error, info};
use urlencoding::encode;

#[async_trait]
impl<T> RainbowSSIAuthProviderManagerTrait for Manager<T>
where
    T: AuthRepoFactoryTrait + Send + Sync + Clone + 'static,
{
    async fn generate_uri(&self, ver_model: auth_verification::Model) -> anyhow::Result<String> {
        info!("Generating verification exchange URI");

        let provider_url = self.config.get_ssi_auth_host_url().unwrap(); // ALWAYS EXPECTED
        let provider_url = match self.config.get_environment_scenario() {
            true => provider_url.replace("127.0.0.1", "host.docker.internal"),
            false => provider_url,
        };
        let provider_url = format!("{}/api/v1", provider_url);

        let base_url = "openid4vp://authorize";

        let encoded_client_id = encode(&ver_model.audience);

        let presentation_definition_uri = format!("{}/pd/{}", &provider_url, ver_model.state);
        let encoded_presentation_definition_uri = encode(&presentation_definition_uri);

        let response_uri = format!("{}/verify/{}", &provider_url, ver_model.state);
        let encoded_response_uri = encode(&response_uri);

        let response_type = "vp_token";
        let response_mode = "direct_post";
        let client_id_scheme = "redirect_uri";

        // TODO let client_metadata = r#"{"authorization_encrypted_response_alg":"ECDH-ES","authorization_encrypted_response_enc":"A256GCM"}"#;

        let uri = format!("{}?response_type={}&client_id={}&response_mode={}&presentation_definition_uri={}&client_id_scheme={}&nonce={}&response_uri={}", base_url, response_type, encoded_client_id, response_mode, encoded_presentation_definition_uri, client_id_scheme, ver_model.nonce, encoded_response_uri);
        info!("Uri generated successfully: {}", uri);

        Ok(uri)
    }
    async fn manage_access(&self, payload: GrantRequest) -> anyhow::Result<GrantResponse> {
        info!("Managing access");

        let interact = match payload.interact {
            Some(model) => model,
            None => {
                let error = CommonErrors::not_impl_new(
                    "Only petitions with an 'interact field' are supported right now".to_string(),
                    Some("Only petitions with an 'interact field' are supported right now".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        if !&interact.start.contains(&"oidc4vp".to_string()) {
            let error = CommonErrors::not_impl_new(
                "Interact method not supported yet".to_string(),
                Some("Interact method not supported yet".to_string()),
            );
            error!("{}", error.log());
            bail!(error);
        }

        let provider_url = self.config.get_ssi_auth_host_url().unwrap(); //  EXPECTED ALWAYS
        let provider_url = format!("{}/api/v1", provider_url);
        let host_url = match self.config.get_environment_scenario() {
            true => provider_url.clone().replace("127.0.0.1", "host.docker.internal"),
            false => provider_url.clone(),
        };

        let client_id = format!("{}/verify", &host_url);

        let grant_endpoint = format!("{}/access", provider_url);

        let id = uuid::Uuid::new_v4().to_string();

        let client = payload.client;
        let class_id = match client["class_id"].as_str() {
            Some(data) => data.to_string(),
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("Missing field class_id in the petition".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let new_request_model = auth_request::NewModel { id: id.clone(), consumer_slug: class_id };

        let _ = match self.repo.request().create(new_request_model).await {
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

        let continue_endpoint = format!("{}/continue", &provider_url);
        let continue_token = create_opaque_token();
        let new_interaction_model = auth_interaction::NewModel {
            id: id.clone(),
            start: interact.start,
            method: interact.finish.method,
            uri: interact.finish.uri.unwrap(), // EXPECTED ALWAYS
            client_nonce: interact.finish.nonce,
            hash_method: interact.finish.hash_method,
            hints: interact.hints,
            grant_endpoint,
            continue_endpoint,
            continue_token,
        };

        let interaction_model = match self.repo.interaction().create(new_interaction_model).await {
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
            r#type: payload.access_token.access.r#type,
            actions: payload.access_token.access.actions.unwrap_or(vec![String::from("talk")]),
            locations: payload.access_token.access.locations,
            datatypes: payload.access_token.access.datatypes,
            identifier: payload.access_token.access.identifier,
            privileges: payload.access_token.access.privileges,
            label: payload.access_token.label,
            flags: payload.access_token.flags,
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

        let new_verification_model = auth_verification::NewModel { id: id.clone(), audience: client_id };

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

        let uri = self.generate_uri(verification_model).await?;

        let response = GrantResponse::default4oidc4vp(
            interaction_model.id,
            interaction_model.continue_endpoint,
            interaction_model.continue_token,
            interaction_model.as_nonce,
            uri,
        );
        Ok(response)
    }

    async fn validate_continue_request(
        &self,
        cont_id: String,
        interact_ref: String,
        token: String,
    ) -> anyhow::Result<auth_interaction::Model> {
        info!("Validating continue request");
        let int_model = match self.repo.interaction().get_by_cont_id(cont_id.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    cont_id.to_string(),
                    Some(format!("There is no process with cont_id: {}", &cont_id)),
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

        if interact_ref != int_model.interact_ref {
            let error = AuthErrors::security_new(Some(format!(
                "Interact reference '{}' does not match '{}'",
                interact_ref, int_model.interact_ref
            )));
            error!("{}", error.log());
            bail!(error);
        }

        if token != int_model.continue_token {
            let error = AuthErrors::security_new(Some(format!(
                "Token '{}' does not match '{}'",
                token, int_model.continue_token
            )));
            error!("{}", error.log());
            bail!(error);
        }
        Ok(int_model)
    }

    async fn continue_req(&self, int_model: auth_interaction::Model) -> anyhow::Result<auth_request::Model> {
        let id = int_model.clone().id;
        let mut request_model = match self.repo.request().get_by_id(id.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    id.clone(),
                    Some(format!("There is no process with cont_id: {}", &id)),
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

        let token = create_opaque_token();
        request_model.token = Some(token);
        request_model.status = "Approved".to_string();

        let new_request_model = match self.repo.request().update(request_model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        // if model.status != "pending" {
        //     bail!("Too many attempts"); // TODO
        // }

        Ok(new_request_model)
    }

    async fn retrieve_data(
        &self,
        req_model: auth_request::Model,
        int_model: auth_interaction::Model,
    ) -> anyhow::Result<mates::NewModel> {
        let id = int_model.id;
        let ver_model = match self.repo.verification().get_by_id(id.as_str()).await {
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

        let base_url = Some(trim_4_base(int_model.uri.as_str()));
        let mate = mates::NewModel {
            participant_id: ver_model.holder.unwrap(), // EXPECTED ALWAYS
            participant_slug: req_model.consumer_slug,
            participant_type: "Agent".to_string(),
            base_url,
            token: req_model.token,
            is_me: false,
        };
        Ok(mate)
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

    async fn generate_vp_def(&self, state: String) -> anyhow::Result<Value> {
        let model = match self.repo.verification().get_by_state(state.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    state.clone(),
                    Some(format!("There is no process with state: {}", &state)),
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

        Ok(json!({
          "id": model.id,
          "input_descriptors": [
            {
              "id": "DataspaceParticipantCredential",
              "format": {
                "jwt_vc_json": {
                  "alg": [
                    "RSA"
                  ]
                }
              },
              "constraints": {
                "fields": [
                  {
                    "path": [
                      "$.vc.type"
                    ],
                    "filter": {
                      "type": "string",
                      "pattern": "DataspaceParticipantCredential"
                    }
                  }
                ]
              }
            }
          ]
        }))
    }

    async fn verify_all(&self, state: String, vp_token: String) -> anyhow::Result<String> {
        let verification_model = match self.repo.verification().get_by_state(state.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    state.clone(),
                    Some(format!("There is no process with state: {}", &state)),
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

        let (vcts, holder) = match self.verify_vp(verification_model.clone(), vp_token).await {
            Ok((vcts, holder)) => (vcts, holder),
            Err(e) => {
                let mut new_model = match self.repo.verification().get_by_id(verification_model.id.as_str()).await {
                    Ok(Some(model)) => model,
                    Ok(None) => {
                        let error = CommonErrors::missing_resource_new(
                            verification_model.id.clone(),
                            Some(format!(
                                "There is no process with id: {}",
                                &verification_model.id
                            )),
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

                new_model.success = Some(false);
                new_model.ended_at = Some(Utc::now().naive_utc());
                match self.repo.verification().update(new_model).await {
                    Ok(_) => {}
                    Err(e) => {
                        let error = CommonErrors::database_new(Some(e.to_string()));
                        error!("{}", error.log());
                        bail!(error);
                    }
                };
                bail!(e)
            }
        };

        for cred in vcts {
            match self.verify_vc(cred, holder.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    let mut new_model = match self.repo.verification().get_by_id(verification_model.id.as_str()).await {
                        Ok(Some(model)) => model,
                        Ok(None) => {
                            let error = CommonErrors::missing_resource_new(
                                verification_model.id.clone(),
                                Some(format!(
                                    "There is no process with id: {}",
                                    &verification_model.id
                                )),
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
                    new_model.success = Some(false);
                    new_model.ended_at = Some(Utc::now().naive_utc());
                    match self.repo.verification().update(new_model).await {
                        Ok(_) => {}
                        Err(e) => {
                            let error = CommonErrors::database_new(Some(e.to_string()));
                            error!("{}", error.log());
                            bail!(error);
                        }
                    };
                    bail!(e)
                }
            }
        }
        info!("VP & VC Validated successfully");

        let mut new_request_model = self.repo.request().get_by_id(verification_model.id.as_str()).await?.unwrap();
        let mut new_ver_model = self.repo.verification().get_by_id(verification_model.id.as_str()).await?.unwrap();

        new_ver_model.ended_at = Some(Utc::now().naive_utc());
        new_request_model.status = "Processing".to_string();

        match self.repo.request().update(new_request_model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };
        match self.repo.verification().update(new_ver_model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        Ok(verification_model.id)
    }

    async fn verify_vp(
        &self,
        model: auth_verification::Model,
        vp_token: String,
    ) -> anyhow::Result<(Vec<String>, String)> {
        info!("Verifying VP");
        let header = jsonwebtoken::decode_header(&vp_token)?;
        let kid_str = match header.kid.as_ref() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("Jwt does not contain a token".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        // let (kid, kid_id) = split_did(kid_str.as_str()); // TODO KID_ID
        let (kid, _) = split_did(kid_str.as_str()); // TODO KID_ID
        let alg = header.alg;

        let vec = URL_SAFE_NO_PAD.decode(&(kid.replace("did:jwk:", "")))?;
        let jwk: Jwk = serde_json::from_slice(&vec)?;

        let key = jsonwebtoken::DecodingKey::from_jwk(&jwk)?;
        let audience = format!(
            "{}/api/v1/verify/{}",
            self.config.get_ssi_auth_host_url().unwrap(), // EXPECTED ALWAYS
            &model.state
        );

        let audience = match self.config.get_environment_scenario() {
            true => audience.replace("127.0.0.1", "host.docker.internal"),
            false => audience,
        };

        let mut val = Validation::new(alg);

        val.required_spec_claims = HashSet::new();
        val.validate_aud = true;
        val.set_audience(&[&(audience)]);
        val.validate_exp = false;
        val.validate_nbf = true;

        let token = match jsonwebtoken::decode::<Value>(&vp_token, &key, &val) {
            Ok(token) => token,
            Err(e) => {
                let error = AuthErrors::security_new(Some(format!(
                    "VPT signature is incorrect -> {}",
                    e.to_string()
                )));
                error!("{}", error.log());
                bail!(error);
            }
        };

        debug!("{:#?}", token);
        info!("VPT token signature is correct");

        // let id = match token.claims["jti"].as_str() {
        //     Some(data) => data,
        //     None => {
        //         let error = CommonErrors::format_new(
        //             BadFormat::Received,
        //             Some("VPT does not contain the 'jti' field".to_string()),
        //         );
        //         error!("{}", error.log());
        //         bail!(error);
        //     }
        // };
        let nonce = match token.claims["nonce"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("VPT does not contain the 'nonce' field".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let sub = match token.claims["sub"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("VPT does not contain the 'sub' field".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        let iss = match token.claims["iss"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("VPT does not contain the 'iss' field".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        if sub != iss || iss != kid {
            // VALIDATE HOLDER 1
            let error = AuthErrors::security_new(Some(
                "VPT token issuer, subject & kid does not match".to_string(),
            ));
            error!("{}", error.log());
            bail!(error);
        }
        info!("VPT issuer, subject & kid matches");

        let mut model = model.clone();
        model.holder = Some(sub.to_string());
        model.vpt = Some(vp_token);

        let new_model = match self.repo.verification().update(model).await {
            Ok(model) => model,
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        if new_model.nonce != nonce {
            // VALIDATE NONCE
            let error = AuthErrors::security_new(Some("Invalid nonce, it does not match".to_string()));
            error!("{}", error.log());
            bail!(error);
        }
        info!("VPT Nonce matches");

        let vp_id = match token.claims["vp"]["id"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("VPT does not contain the 'vp_id' field".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        if new_model.id != vp_id {
            // VALIDATE ID MATCHES JTI
            let error = AuthErrors::security_new(Some("Invalid id, it does not match".to_string()));
            error!("{}", error.log());
            bail!(error);
        }
        info!("Exchange is valid");

        let vp_holder = match token.claims["vp"]["holder"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("VPT does not contain the 'vp_holder' field".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        if new_model.holder.unwrap() != vp_holder {
            // EXPECTED ALWAYS
            let error = AuthErrors::security_new(Some("Invalid holder, it does not match".to_string()));
            error!("{}", error.log());
            bail!(error);
        }
        info!("vp holder matches vpt subject & issuer");
        info!("VP Verification successful");

        let vct: Vec<String> = match serde_json::from_value(token.claims["vp"]["verifiableCredential"].clone()) {
            Ok(data) => data,
            Err(e) => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some(format!(
                        "VPT does not contain the 'verifiableCredential' field -> {}",
                        e.to_string()
                    )),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        Ok((vct, kid.to_string()))
    }

    async fn verify_vc(&self, vc_token: String, vp_holder: String) -> anyhow::Result<()> {
        info!("Verifying VC");
        let header = jsonwebtoken::decode_header(&vc_token)?;

        let kid_str = match header.kid.as_ref() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("Jwt does not contain a token".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        // let (kid, kid_id) = split_did(kid_str.as_str()); // TODO KID_ID
        let (kid, _) = split_did(kid_str.as_str()); // TODO KID_ID
        let alg = header.alg;

        let vec = URL_SAFE_NO_PAD.decode(&(kid.replace("did:jwk:", "")))?; // TODO
        let jwk: Jwk = serde_json::from_slice(&vec)?;

        let key = jsonwebtoken::DecodingKey::from_jwk(&jwk)?;

        let mut val = Validation::new(alg);
        val.required_spec_claims = HashSet::new();
        val.validate_aud = false;
        val.validate_exp = false; // TODO by now, the VCs do not expire
        val.validate_nbf = true;

        let token = match jsonwebtoken::decode::<Value>(&vc_token, &key, &val) {
            Ok(token) => token,
            Err(e) => {
                let error = CommonErrors::format_new(BadFormat::Received, Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        debug!("{:#?}", token);

        info!("VCT token signature is correct");

        let iss = match token.claims["iss"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(BadFormat::Received, Some("No issuer in the vc".to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };
        let vc_iss_id = match token.claims["vc"]["issuer"]["id"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No issuer id in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        if iss != kid || kid != vc_iss_id {
            // VALIDATE IF ISSUER IS THE SAME AS KID
            let error = AuthErrors::security_new(Some("VCT token issuer & kid does not match".to_string()));
            error!("{}", error.log());
            bail!(error);
        }
        info!("VCT issuer & kid matches");

        // if issuers_list.contains(kid) {
        //     // TODO
        //     error!("VCT issuer is not on the trusted issuers list");
        //     bail!("VCT issuer is not on the trusted issuers list");
        // }
        // info!("VCT issuer is on the trusted issuers list");

        let sub = match token.claims["sub"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No sub field in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let cred_sub_id = match token.claims["vc"]["credentialSubject"]["id"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No credentialSubject id field in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        if sub != &vp_holder || &vp_holder != cred_sub_id {
            let error = AuthErrors::security_new(Some(
                "VCT token sub, credential subject & VP Holder do not match".to_string(),
            ));
            error!("{}", error.log());
            bail!(error);
        }
        info!("VC Holder Data is Correct");

        let jti = match token.claims["jti"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No jti id field in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        let vc_id = match token.claims["vc"]["id"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No vc_id id field in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        if jti != vc_id {
            let error = AuthErrors::security_new(Some("VCT jti & VC id do not match".to_string()));
            error!("{}", error.log());
            bail!(error);
        }
        info!("VCT jti & VC id match");

        let valid_from = match token.claims["vc"]["validFrom"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No validFrom id field in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };
        match DateTime::parse_from_rfc3339(valid_from) {
            Ok(parsed_date) => {
                if parsed_date > Utc::now() {
                    let error = AuthErrors::security_new(Some("VC is not valid yet".to_string()));
                    error!("{}", error.log());
                    bail!(error)
                }
            }
            Err(e) => {
                let error = AuthErrors::security_new(Some(format!(
                    "VC iat and issuanceDate do not match -> {}",
                    e
                )));
                error!("{}", error.log());
                bail!(error);
            }
        };

        info!("VC validFrom is correct");
        let valid_until = match token.claims["vc"]["validUntil"].as_str() {
            Some(data) => data,
            None => {
                let error = CommonErrors::format_new(
                    BadFormat::Received,
                    Some("No validUntil field in the vc".to_string()),
                );
                error!("{}", error.log());
                bail!(error);
            }
        };

        match DateTime::parse_from_rfc3339(valid_until) {
            Ok(parsed_date) => {
                if Utc::now() > parsed_date {
                    let error = AuthErrors::security_new(Some("VC has expired".to_string()));
                    error!("{}", error.log());
                    bail!(error)
                }
            }
            Err(e) => {
                let error = AuthErrors::security_new(Some(format!("VC validUntil has invalid format -> {}", e)));
                error!("{}", error.log());
                bail!(error);
            }
        }

        info!("VC validUntil is correct");
        info!("VC Verification successful");
        Ok(())
    }

    async fn end_verification(&self, id: String) -> anyhow::Result<Option<String>> {
        match self.repo.interaction().get_by_id(id.as_str()).await {
            Ok(Some(model)) => {
                if model.method == "redirect" {
                    let redirect_uri = format!(
                        "{}?hash={}&interact_ref={}",
                        model.uri, model.hash, model.interact_ref
                    );
                    Ok(Some(redirect_uri))
                } else if model.method == "push" {
                    // TODO
                    let url = model.uri;

                    let mut headers = HeaderMap::new();
                    headers.insert(CONTENT_TYPE, "application/json".parse()?);
                    headers.insert(ACCEPT, "application/json".parse()?);

                    let body = CallbackBody { interact_ref: model.interact_ref, hash: model.hash };

                    let _ = match self.client.post(&url).headers(headers).json(&body).send().await {
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
                    Ok(None)
                } else {
                    let error = CommonErrors::not_impl_new(
                        "Interact method not supported".to_string(),
                        Some(format!("Interact method {} not supported", model.method)),
                    );
                    error!("{}", error.log());
                    bail!(error);
                }
            }
            Ok(None) => {
                let verification_model = self.repo.verification().get_by_id(id.as_str()).await?.unwrap();
                let token = create_opaque_token();
                let bus_mate_model = business_mates::NewModel {
                    id: verification_model.state,
                    participant_id: verification_model.holder.unwrap(),
                    token: Some(token),
                };
                self.repo.business_mates().create(bus_mate_model).await?;
                Ok(None)
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        }
    }

    async fn fast_login(&self, state: String) -> anyhow::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let nonce: String = rand::thread_rng().sample_iter(&Alphanumeric).take(12).map(char::from).collect();
        let provider_url = format!("{}/api/v1", self.config.get_ssi_auth_host_url().unwrap()); //  EXPECTED ALWAYS
        let provider_url = match self.config.get_environment_scenario() {
            true => provider_url.replace("127.0.0.1", "host.docker.internal"),
            false => provider_url,
        };

        let client_id = format!("{}/verify", &provider_url);
        let audience = format!("{}/{}", client_id, &state);
        let new_ver_model = auth_verification::Model {
            id: id.clone(),
            state,
            nonce,
            audience,
            holder: None,
            vpt: None,
            success: None,
            status: "Pending".to_string(),
            created_at: Utc::now().naive_utc(),
            ended_at: None,
        };

        let ver_model = match self.repo.verification().create_extra(new_ver_model).await {
            Ok(model) => {
                info!("Verification model saved successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let new_req_model = auth_request::NewModel { id, consumer_slug: "--".to_string() };
        let _ = match self.repo.request().create(new_req_model).await {
            Ok(model) => {
                info!("Request authentication saved successfully");
                model
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };

        let uri = self.generate_uri(ver_model).await?;
        Ok(uri)
    }

    async fn verify_token(&self, token: String) -> anyhow::Result<mates::Model> {
        info!("Validating token");

        let model = match self.repo.mates().get_by_token(token.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::unauthorized_new(Some(format!("Invalid token: {}", &token)));
                error!("{}", error.log());
                bail!(error);
            }
            Err(e) => {
                let error = CommonErrors::database_new(Some(e.to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };
        Ok(model)
    }

    async fn retrieve_business_token(&self, id: String) -> anyhow::Result<Value> {
        let model = match self.repo.business_mates().get_by_id(id.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_resource_new(
                    id.clone(),
                    Some(format!("There is process with id: {}", &id)),
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

        let mate = match self.repo.mates().get_by_id(model.participant_id.as_str()).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                let error = CommonErrors::missing_action_new(
                    "Onboarding".to_string(),
                    MissingAction::Onboarding,
                    Some("Onboarding is a requisite to access this service".to_string()),
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

        let token = match model.token {
            Some(token) => token,
            None => {
                let error = CommonErrors::unauthorized_new(Some("He does not have a token".to_string()));
                error!("{}", error.log());
                bail!(error);
            }
        };
        let response = json!({
            "token": token,
            "mate": mate
        });

        Ok(response)
    }
}
