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

use crate::services::{HasGateKeeper, HasRepo};
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::grant_response::{ErrorResponse, GrantResponse};
use ymir::types::gnap::GrantStatus;
use ymir::utils::{create_opaque_token, errors_to_error_code, require_field};

#[async_trait]
pub trait GateKeeperModule: HasGateKeeper + HasVerifier + HasRepo + Send + Sync + 'static {
    async fn manage_grant_req(&self, payload: Bytes, headers: HeaderMap) -> GrantResponse {
        self.inner_manage_grant_req(payload, headers)
            .await
            .unwrap_or_else(|e| {
                e.log();
                let code = errors_to_error_code(&e);
                GrantResponse::Error(ErrorResponse { error: code })
            })
    }

    async fn manage_continue_req(
        &self,
        id: String,
        payload: Bytes,
        headers: HeaderMap,
    ) -> GrantResponse {
        self.inner_manage_continue_req(id, payload, headers)
            .await
            .unwrap_or_else(|e| {
                e.log();
                let code = errors_to_error_code(&e);
                GrantResponse::Error(ErrorResponse { error: code })
            })
    }

    // ========================================= INTERNALS =========================================

    async fn inner_manage_grant_req(
        &self,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let grant_request = self.gatekeeper().validate_grant_req(&payload, &headers)?;

        let grant = self
            .gatekeeper()
            .build_grant_plan(grant_request.client.class_id.clone())?;
        let interaction = self.gatekeeper().build_interaction_plan(
            &grant.id,
            grant_request.client,
            grant_request.interact,
        )?;
        let resource_req = self
            .gatekeeper()
            .build_resource_req_plan(&grant.id, grant_request.kind)?;

        let grant = self.repo().recv_grant().create(grant).await?;
        let interaction = self.repo().recv_interaction().create(interaction).await?;
        let _resource_req = self.repo().resource_req().create(resource_req).await?;

        let verification = self.verifier().build_vp_plan(&grant.id)?;
        let ver_model = self.repo().recv_verification().create(verification).await?;
        let uri = self.verifier().generate_verification_uri(&ver_model);
        Ok(GrantResponse::pending(uri, &interaction))
    }

    async fn inner_manage_continue_req(
        &self,
        id: String,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let interaction = self.repo().recv_interaction().get_by_cont_id(&id).await?;
        self.gatekeeper()
            .validate_cont_req(&interaction, &payload, &headers)?;

        let mut grant = self.repo().recv_grant().get_by_id(&interaction.id).await?;
        let verification = self
            .repo()
            .recv_verification()
            .get_by_id(&interaction.id)
            .await?;

        let holder = require_field(verification.holder.as_ref(), "holder")?;
        let token = create_opaque_token();

        let mate = self.gatekeeper().build_mate_plan(
            holder,
            &grant.participant_nick,
            &interaction.callback_uri,
            &token,
        );

        let _mate = self.repo().participant().force_update(mate).await?;

        grant.token = Some(token.to_string());
        grant.status = GrantStatus::Approved;

        let _grant = self.repo().recv_grant().update(grant).await?;

        let resource_req = self
            .repo()
            .resource_req()
            .get_by_id(&interaction.id)
            .await?;

        Ok(GrantResponse::token_approved(token, &resource_req))
    }
}
