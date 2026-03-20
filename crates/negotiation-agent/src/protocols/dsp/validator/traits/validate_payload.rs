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
use common::config::types::roles::RoleConfig;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ValidatePayload: Send + Sync + 'static {
    /// Validates with json schema
    async fn validate_with_json_schema(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<()>;
    /// Validates uri in URL to check if it is URN encoded
    async fn validate_uri_id_as_urn(&self, uri_id: &String) -> Outcome<()>;
    /// Validates if identifiers provider_pid and consumer_pid are urn
    async fn validate_identifiers_as_urn(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<()>;
    /// Validates depending on role if uri_id == ***_pid
    async fn validate_uri_and_pid(
        &self,
        uri_id: &String,
        payload: &dyn NegotiationProcessMessageTrait,
        role: &RoleConfig,
    ) -> Outcome<()>;
    /// Validates if consumer_pid and provider_pid are equal to identifiers in db
    async fn validate_correlation(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
        dto: &NegotiationProcessDto,
    ) -> Outcome<()>; // db call
    /// Validates if Header Bearer token corresponds to associated_consumer in db
    async fn validate_auth(&self, payload: &dyn NegotiationProcessMessageTrait) -> Outcome<()>; // db call
    /// Validates if data_address_present if format contains PUSH
    async fn validate_format_data_address(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<()>;
    /// Validates if data_address_present if format contains PUSH
    async fn validate_data_address_in_start(
        &self,
        payload: &dyn NegotiationProcessMessageTrait,
        dto: &NegotiationProcessDto,
    ) -> Outcome<()>;
}
