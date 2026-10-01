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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::protocols::dsp::protocol_types::CatalogMessageTrait;
use crate::protocols::dsp::validator::traits::validate_payload::ValidatePayload;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use common::config::types::roles::RoleConfig;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ValidatePayloadService {
    _helpers: Arc<dyn ValidationHelpers>,
}
impl ValidatePayloadService {
    pub fn new(_helpers: Arc<dyn ValidationHelpers>) -> Self {
        Self { _helpers }
    }
}
#[async_trait::async_trait]
impl ValidatePayload for ValidatePayloadService {
    async fn validate_with_json_schema(&self, payload: &dyn CatalogMessageTrait) -> Outcome<()> {
        todo!()
    }

    async fn validate_uri_id_as_urn(&self, uri_id: &String) -> Outcome<()> {
        todo!()
    }

    async fn validate_identifiers_as_urn(&self, payload: &dyn CatalogMessageTrait) -> Outcome<()> {
        todo!()
    }

    async fn validate_uri_and_pid(
        &self,
        uri_id: &String,
        payload: &dyn CatalogMessageTrait,
        role: &RoleConfig,
    ) -> Outcome<()> {
        todo!()
    }

    async fn validate_auth(&self, payload: &dyn CatalogMessageTrait) -> Outcome<()> {
        todo!()
    }
}
