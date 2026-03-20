/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::protocol_types::NegotiationProcessMessageTrait;
use crate::protocols::dsp::validator::traits::validate_payload::ValidatePayload;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use common::config::types::roles::RoleConfig;
use std::str::FromStr;
use std::sync::Arc;
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
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<()> {
        // TODO set json_schema
        Ok(())
    }

    async fn validate_uri_id_as_urn(&self, uri_id: &String) -> Outcome<()> {
        self.helpers.parse_urn(uri_id).await?;
        Ok(())
    }

    #[allow(unused)]
    async fn validate_identifiers_as_urn(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<()> {
        // Are as urn defined in dtos
        Ok(())
    }

    async fn validate_uri_and_pid(
        &self,
        uri_id: &String,
        payload: &dyn NegotiationProcessMessageTrait,
        role: &RoleConfig,
    ) -> Outcome<()> {
        let identifier = match role {
            RoleConfig::Provider => payload.get_provider_pid(),
            RoleConfig::Consumer => payload.get_consumer_pid(),
            _ => {
                return Err(Errors::parse(
                    "Something went wrong. Role not recognized.",
                    None,
                ));
            }
        }
        .ok_or_else(|| Errors::parse("Something went wrong. Role not recognized.", None))?
        .to_string();
        let uri_id = self.helpers.parse_urn(uri_id).await?.to_string();
        if identifier.ne(&uri_id) {
            return Err(Errors::parse(
                "Uri string and body identifier are not correlated",
                None,
            ));
        }
        Ok(())
    }

    async fn validate_correlation(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
        dto: &NegotiationProcessDto,
    ) -> Outcome<()> {
        let provider_pid_in_dto = self
            .helpers
            .get_pid_by_role(dto, &RoleConfig::Provider)
            .await?
            .to_string();
        let consumer_pid_in_dto = self
            .helpers
            .get_pid_by_role(dto, &RoleConfig::Consumer)
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
            return Err(Errors::parse(
                "Uri string and body identifier are not correlated",
                None,
            ));
        }
        Ok(())
    }

    #[allow(unused)]
    async fn validate_auth(&self, payload: &dyn NegotiationProcessMessageTrait) -> Outcome<()> {
        // TODO
        Ok(())
    }

    async fn validate_format_data_address(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn validate_data_address_in_start(
        &self,
        _payload: &dyn NegotiationProcessMessageTrait,
        _dto: &NegotiationProcessDto,
    ) -> Outcome<()> {
        Ok(())
    }
}
