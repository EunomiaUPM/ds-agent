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

use crate::entities::filters::RecvGrantFilter;
use crate::services::{HasGateKeeper, HasRepo};
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use common::facades::grants_facade::VerifiedPeer;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::received::grant;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::grant_response::{ErrorResponse, GrantResponse};
use ymir::types::gnap::GrantStatus;
use ymir::types::listing::{GrantSort, RecvGrantListFilter};
use ymir::types::oauth::{RoleTrait, UserInfo};
use ymir::utils::{create_opaque_token, errors_to_error_code, require_field};

/// Answering GNAP grant requests from peers: each one is verified with an OID4VP presentation.
///
/// A received grant has no author, only the role that handles it and its visibility: users of
/// that role or above handle it, and anyone sees it if it is public.
#[async_trait]
pub trait GateKeeperModule: HasGateKeeper + HasVerifier + HasRepo + Send + Sync + 'static {
    // ==========================================================================================
    // Received grants: queries
    // ==========================================================================================

    /// Page of the received grants `user` sees.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filter: &RecvGrantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<grant::Model>> {
        let list_page = page.list_page(sort, GrantSort::Created, GrantSort::Updated)?;
        let list_filter = RecvGrantListFilter {
            role: user.role().clone(),
            kind: filter.kind.clone().unwrap_or(GrantKind::AccessToken),
            nick_contains: filter.participant_nick.clone(),
            status: filter.status.clone(),
            created_after: filter.created_after,
            created_before: filter.created_before,
        };
        let listed = self
            .repo()
            .recv_grant()
            .find_page(&list_filter, &list_page)
            .await?;
        let sort_field = list_page.sort;
        Ok(Paginated::from_page(
            listed.items,
            &page.clamped(),
            Some(listed.total),
            |last| {
                let ts = match sort_field {
                    GrantSort::Created => last.created_at,
                    GrantSort::Updated => last.ended_at.unwrap_or(last.created_at),
                };
                Cursor::encode_composite(&ts, &last.id)
            },
        ))
    }

    /// The received grant `id` if `user` sees it; missing-resource error otherwise.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id(&self, user: &UserInfo, id: &str) -> Outcome<grant::Model> {
        let grant = self.repo().recv_grant().get_by_id(id).await?;
        user.ensure_sees_team(&grant.role, &grant.visibility, id)?;
        Ok(grant)
    }

    /// The received grant with its resource request, interaction and verification, if `user`
    /// sees the grant.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id_with_details(&self, user: &UserInfo, id: &str) -> Outcome<Value> {
        let grant = self.get_by_id(user, id).await?;
        let resource_req = self.repo().resource_req().get_by_id(id).await?;
        let interaction = self.repo().recv_interaction().get_by_id(id).await.ok();
        let verification = self.repo().recv_verification().get_by_id(id).await.ok();
        Ok(json!({
            "grant": grant,
            "resource_req": resource_req,
            "interaction": interaction,
            "verification": verification,
        }))
    }

    // ==========================================================================================
    // Received grants: for the other agents (grants facade)
    // ==========================================================================================

    /// The peer behind `token` and the role that handles what it opens, if this connector
    /// issued the token and the grant is approved; missing-resource error otherwise. No user:
    /// the caller is a DSP endpoint answering the peer.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn verify_token(&self, token: &str) -> Outcome<VerifiedPeer> {
        let grant = self.repo().recv_grant().get_approved_by_token(token).await?;
        let participant_id = require_field(grant.participant_id.as_ref(), "participant_id")?;
        Ok(VerifiedPeer {
            participant_id: participant_id.to_string(),
            role: grant.role,
            visibility: grant.visibility,
        })
    }

    // ==========================================================================================
    // Flow entry points, called by the routers (peers, authenticated by GNAP)
    // ==========================================================================================

    /// Starts a grant and answers with the verification URI; failures become a GNAP error body.
    #[tracing::instrument(level = "info", skip_all)]
    async fn manage_grant_req(&self, payload: Bytes, headers: HeaderMap) -> GrantResponse {
        self.inner_manage_grant_req(payload, headers)
            .await
            .unwrap_or_else(|e| {
                e.log();
                let code = errors_to_error_code(&e);
                GrantResponse::Error(ErrorResponse { error: code })
            })
    }

    /// Continues a verified grant and issues the peer's access token; failures become a GNAP
    /// error body.
    #[tracing::instrument(level = "info", skip_all)]
    async fn manage_continue_req(
        &self,
        id: &str,
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

    // ==========================================================================================
    // Internal steps of the flow
    // ==========================================================================================

    /// Validates the request and stores the grant, its interaction, resource request and
    /// verification. The grant's role and visibility would come from rules on what the peer asks
    /// for; until those exist, the root handles it and it is public.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn inner_manage_grant_req(
        &self,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let grant_request = self.gatekeeper().validate_grant_req(&payload, &headers)?;

        let grant =
            self.gatekeeper()
                .build_grant_plan(None, None, grant_request.client.class_id.clone())?;
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
        self.repo().resource_req().create(resource_req).await?;

        let verification = self.verifier().build_vp_plan(&grant.id)?;
        let verification = self.repo().recv_verification().create(verification).await?;
        let uri = self.verifier().generate_verification_uri(&verification);
        Ok(GrantResponse::pending(uri, &interaction))
    }

    /// Checks the continuation against its interaction (the peer proves its key) and, with the
    /// presentation verified, approves the grant with a new token. Stores the peer (if new; an
    /// existing one is left as it is), the private relation of the verification with it, and
    /// which peer the grant belongs to.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn inner_manage_continue_req(
        &self,
        id: &str,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let interaction = self.repo().recv_interaction().get_by_cont_id(id).await?;
        self.gatekeeper()
            .validate_cont_req(&interaction, &payload, &headers)?;

        let mut grant = self.repo().recv_grant().get_by_id(&interaction.id).await?;
        let verification = self
            .repo()
            .recv_verification()
            .get_by_id(&interaction.id)
            .await?;
        let holder = require_field(verification.holder.as_ref(), "holder")?;

        let mate = self.gatekeeper().build_mate_plan(
            holder,
            &grant.participant_nick,
            &interaction.callback_uri,
        );
        self.repo().participant().create_if_absent(mate).await?;
        let relation = self.gatekeeper().build_mate_rel_plan(&grant.role, holder);
        self.repo().participant_relation().force_update(relation).await?;

        let token = create_opaque_token();
        grant.token = Some(token.to_string());
        grant.participant_id = Some(holder.to_string());
        grant.status = GrantStatus::Approved;
        self.repo().recv_grant().update(grant).await?;

        let resource_req = self
            .repo()
            .resource_req()
            .get_by_id(&interaction.id)
            .await?;
        Ok(GrantResponse::token_approved(token, &resource_req))
    }
}
