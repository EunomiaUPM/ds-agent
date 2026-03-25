/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::protocol_types::{TransferProcessMessageTrait, TransferStateAttribute};
use crate::protocols::dsp::validator::traits::validate_payload::ValidatePayload;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use common::config::types::roles::RoleConfig;
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct ValidatePayloadService {
    helpers: Arc<dyn ValidationHelpers>,
}
impl ValidatePayloadService {
    pub fn new(helpers: Arc<dyn ValidationHelpers>) -> Self {
        Self { helpers }
    }
}
#[async_trait::async_trait]
impl ValidatePayload for ValidatePayloadService {
    #[allow(unused)]
    async fn validate_with_json_schema(
        &self,
        payload: &dyn TransferProcessMessageTrait,
    ) -> Outcome<()> {
        // TODO set json_schema
        Ok(())
    }

    async fn validate_uri_id_as_urn(&self, uri_id: &String) -> Outcome<()> {
        self.helpers.parse_urn(uri_id).await.map_err(|e| {
            let msg = format!("Uri id parameter must be urn. {}", e);
            error!("{}", msg);
            Errors::parse(msg, None)
        })?;
        Ok(())
    }

    #[allow(unused)]
    async fn validate_identifiers_as_urn(
        &self,
        payload: &dyn TransferProcessMessageTrait,
    ) -> Outcome<()> {
        // Are as urn defined in dtos
        Ok(())
    }

    async fn validate_uri_and_pid(
        &self,
        uri_id: &String,
        payload: &dyn TransferProcessMessageTrait,
        role: &RoleConfig,
    ) -> Outcome<()> {
        let identifier = match role {
            RoleConfig::Provider => payload.get_provider_pid(),
            RoleConfig::Consumer => payload.get_consumer_pid(),
            _ => {
                let err = CommonErrors::parse_new("Something went wrong. Role not recognized.");
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string(), None));
            }
        }
        .ok_or_else(|| {
            let err = CommonErrors::parse_new("Something went wrong. Role not recognized.");
            error!("{}", err.log());
            Errors::parse(err.to_string(), None)
        })?
        .to_string();
        let uri_id = self.helpers.parse_urn(uri_id).await?.to_string();
        if identifier.ne(&uri_id) {
            let err = CommonErrors::parse_new("Uri string and body identifier are not correlated");
            error!("{}", err.log());
            return Err(Errors::parse(err.to_string(), None));
        }
        Ok(())
    }

    async fn validate_correlation(
        &self,
        payload: &dyn TransferProcessMessageTrait,
        dto: &TransferProcessDto,
    ) -> Outcome<()> {
        let provider_pid_in_dto = self
            .helpers
            .get_pid_by_role(dto, RoleConfig::Provider)
            .await?
            .to_string();
        let consumer_pid_in_dto = self
            .helpers
            .get_pid_by_role(dto, RoleConfig::Consumer)
            .await?
            .to_string();
        let provider_pid_in_payload = payload
            .get_provider_pid()
            .unwrap_or(Urn::from_str("urn:fake:0")?)
            .to_string();
        let consumer_pid_in_payload = payload
            .get_consumer_pid()
            .unwrap_or(Urn::from_str("urn:fake:0")?)
            .to_string();
        if provider_pid_in_dto != provider_pid_in_payload
            || consumer_pid_in_dto != consumer_pid_in_payload
        {
            let err = CommonErrors::parse_new("Uri string and body identifier are not correlated");
            error!("{}", err.log());
            return Err(Errors::parse(err.to_string(), None));
        }
        Ok(())
    }

    #[allow(unused)]
    async fn validate_auth(&self, payload: &dyn TransferProcessMessageTrait) -> Outcome<()> {
        // TODO
        Ok(())
    }

    async fn validate_data_address_in_start(
        &self,
        payload: &dyn TransferProcessMessageTrait,
        dto: &TransferProcessDto,
    ) -> Outcome<()> {
        let role = dto
            .inner
            .role
            .parse::<RoleConfig>()
            .map_err(|e| Errors::crazy(format!("Not able to parse RoleConfig: {e}"), None))?;
        let state_attribute = dto
            .inner
            .state_attribute
            .clone()
            .unwrap_or("".to_string())
            .parse::<TransferStateAttribute>()
            .map_err(|e| {
                Errors::crazy(
                    format!("Not able to parse TransferStateAttribute: {e}"),
                    None,
                )
            })?;
        let is_data_address_in_payload = payload.get_data_address().is_some();

        if is_data_address_in_payload == true {
            if role == RoleConfig::Consumer {
                if state_attribute != TransferStateAttribute::OnRequest {
                    return Err(Errors::crazy("Data address should be defined only in the first TransferStart message from provider", None));
                }
            }
        }

        Ok(())
    }
}
