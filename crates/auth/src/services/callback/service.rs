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

use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use reqwest::header::AUTHORIZATION;
use reqwest::Response;
use sha2::{Digest, Sha256, Sha384, Sha512};
use tracing::info;
use ymir::capabilities::HttpSig;
use ymir::data::entities::sent::{grant, interaction};
use ymir::errors::{Errors, Outcome};
use ymir::services::client::ClientTrait;
use ymir::services::vault::{VaultService, VaultTrait};
use ymir::types::gnap::grant_request::interact::HashMethod;
use ymir::types::gnap::grant_request::GrantRequestKind;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{ApprovedCallbackBody, ContinueRequest};
use ymir::types::http::HttpBody;
use ymir::types::keys::{Certificate, PrivateKey};
use ymir::types::secrets::{PemHelper, StringHelper};
use ymir::utils::{
    expect_from_env, http_client, json_headers, require_field, ParseHeaderExt, ResponseExt,
};

use crate::services::callback::CallbackTrait;

pub struct BasicCallbackService {
    vault: Arc<VaultService>,
}

impl BasicCallbackService {
    pub fn new(vault: Arc<VaultService>) -> BasicCallbackService {
        BasicCallbackService { vault }
    }
}

#[async_trait]
impl CallbackTrait for BasicCallbackService {
    fn apply_callback(&self, interaction: &mut interaction::Model, payload: &ApprovedCallbackBody) {
        interaction.interact_ref = Some(payload.interact_ref.clone());
        interaction.hash = Some(payload.hash.clone());
    }
    fn check_callback(
        &self,
        interaction: &interaction::Model,
        grant: &grant::Model,
    ) -> Outcome<()> {
        info!("Checking if callback hash matches expected one");

        let nonce = require_field(interaction.as_nonce.as_ref(), "as_nonce")?;
        let interact_ref = require_field(interaction.interact_ref.as_ref(), "interact_ref")?;
        let hash_input = format!(
            "{}\n{}\n{}\n{}",
            interaction.client_nonce, nonce, interact_ref, grant.grant_endpoint
        );

        let hash_result = match &interaction.hash_method {
            HashMethod::Sha256 => {
                let mut h = Sha256::new();
                h.update(hash_input.as_bytes());
                h.finalize().to_vec()
            }
            HashMethod::Sha384 => {
                let mut h = Sha384::new();
                h.update(hash_input.as_bytes());
                h.finalize().to_vec()
            }
            HashMethod::Sha512 => {
                let mut h = Sha512::new();
                h.update(hash_input.as_bytes());
                h.finalize().to_vec()
            }
            HashMethod::Other(other) => {
                return Err(Errors::not_impl(
                    format!("Hash method '{}' not supported", other),
                    None,
                ));
            }
        };

        let calculated_hash = URL_SAFE_NO_PAD.encode(hash_result);

        let hash = require_field(interaction.hash, "hash")?;
        if calculated_hash != hash {
            return Err(Errors::security(
                "Hash does not match the calculated one",
                None,
            ));
        }

        info!("Hash matches the calculated one");
        Ok(())
    }

    async fn send_continue_req(&self, interaction: &interaction::Model) -> Outcome<GrantResponse> {
        info!("Continuing grant request");

        let url = require_field(interaction.continue_endpoint.as_ref(), "continue-endpoint")?;
        let token = require_field(interaction.continue_token.as_ref(), "continue token")?;

        let cert = expect_from_env("VAULT_APP_CERT");
        let cert: StringHelper = self.vault.read(None, &cert).await?;
        let certificate = Certificate::try_from_pem(cert.data())?;

        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: PemHelper = self.vault.read(None, &priv_key).await?;
        let priv_key = PrivateKey::from_safe_pem(priv_key.pem(), priv_key.kty(), priv_key.crv())?;

        let interact_ref = require_field(interaction.interact_ref.as_ref(), "interact_ref")?;
        let (body, body_bytes) = HttpBody::from_json_bytes(&ContinueRequest {
            interact_ref: interact_ref.clone(),
        })?;

        let authorization = format!("GNAP {}", token);
        let mut headers = json_headers();
        headers.insert(AUTHORIZATION, authorization.parse_header()?);
        let httpsig = HttpSig::build(
            &certificate,
            &priv_key,
            None,
            "POST",
            &url,
            &body_bytes,
            Some(&authorization),
        )?;

        headers.extend(httpsig);

        let res = http_client().post(&url, Some(headers), body).await?;

        res.parse_json().await
    }
}
