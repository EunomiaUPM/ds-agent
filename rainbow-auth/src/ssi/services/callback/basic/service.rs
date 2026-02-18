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
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Response;
use sha2::{Digest, Sha256};
use tracing::{error, info};
use ymir::data::entities::req_interaction;
use ymir::errors::{ErrorLogTrait, Errors};
use ymir::services::client::ClientTrait;
use ymir::types::gnap::{ApprovedCallbackBody, RefBody};
use ymir::types::http::Body;
use ymir::utils::get_from_opt;

use super::super::CallbackTrait;

pub struct BasicCallbackService {
    client: Arc<dyn ClientTrait>,
}

impl BasicCallbackService {
    pub fn new(client: Arc<dyn ClientTrait>) -> BasicCallbackService {
        BasicCallbackService { client }
    }
}

#[async_trait]
impl CallbackTrait for BasicCallbackService {
    fn check_callback(
        &self,
        int_model: &mut req_interaction::Model,
        payload: &ApprovedCallbackBody,
    ) -> anyhow::Result<()> {
        info!("Checking callback");

        int_model.interact_ref = Some(payload.interact_ref.clone());
        int_model.hash = Some(payload.hash.clone());
        let nonce = get_from_opt(&int_model.as_nonce, "as_nonce")?;
        let interact_ref = get_from_opt(&int_model.interact_ref, "interact_ref")?;
        let hash_input = format!(
            "{}\n{}\n{}\n{}",
            int_model.client_nonce, nonce, interact_ref, int_model.grant_endpoint
        );

        let mut hasher = Sha256::new(); // TODO
        hasher.update(hash_input.as_bytes());
        let result = hasher.finalize();

        let calculated_hash = URL_SAFE_NO_PAD.encode(result);

        let hash = get_from_opt(&int_model.hash, "hash")?;
        if calculated_hash != hash {
            let error = Errors::security_new("Hash does not match the calculated one");
            error!("{}", error.log());
            bail!(error);
        }

        info!("Hash matches the calculated one");
        Ok(())
    }

    async fn continue_req(&self, int_model: &req_interaction::Model) -> anyhow::Result<Response> {
        info!("Continuing request");

        let url = get_from_opt(&int_model.continue_endpoint, "continue-endpoint")?;
        let base_token = get_from_opt(&int_model.continue_token, "continue token")?;
        let token = format!("GNAP {}", base_token);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(AUTHORIZATION, token.parse()?);

        let interact_ref = get_from_opt(&int_model.interact_ref, "interact_ref")?;
        let body = RefBody { interact_ref };

        self.client.post(&url, Some(headers), Body::Json(serde_json::to_value(body)?)).await
    }
}
