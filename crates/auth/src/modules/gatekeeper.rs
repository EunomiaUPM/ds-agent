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
use crate::services::{HasGateKeeper, HasRepo, MayHaveEventBus};
use crate::types::events::{recv_owner, GrantEvent};
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use common::facades::grants_facade::VerifiedPeer;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::received::grant;
use ymir::errors::Errors;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::grant_response::{ErrorCode, ErrorResponse, GrantResponse};
use ymir::types::gnap::GrantStatus;
use ymir::types::listing::{GrantSort, RecvGrantListFilter};
use ymir::types::oauth::{RoleTrait, UserInfo};
use ymir::utils::{errors_to_error_code, hash_token, require_field};

/// Answering GNAP grant requests from peers: each one is verified with an OID4VP presentation.
///
/// A received grant has no author, only the role that handles it and its visibility: users of
/// that role or above handle it, and anyone sees it if it is public.
#[async_trait]
pub trait GateKeeperModule:
    HasGateKeeper + HasVerifier + HasRepo + MayHaveEventBus + Send + Sync + 'static
{
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
    /// issued the token, the grant is approved and the token has not expired; missing-resource
    /// error otherwise. No user: the caller is a DSP endpoint answering the peer.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn verify_token(&self, token: &str) -> Outcome<VerifiedPeer> {
        let grant = self
            .repo()
            .recv_grant()
            .get_valid_by_final_hash(&hash_token(token), Utc::now())
            .await?;
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

    #[tracing::instrument(level = "info", skip_all)]
    async fn manage_rotation(
        &self,
        managing_id: &str,
        payload: Bytes,
        headers: HeaderMap,
    ) -> GrantResponse {
        self.inner_manage_rotation(managing_id, payload, headers)
            .await
            .unwrap_or_else(|e| {
                e.log();
                let error = match errors_to_error_code(&e) {
                    server @ ErrorCode::Other(_) => server,
                    _ => ErrorCode::InvalidRotation,
                };
                GrantResponse::Error(ErrorResponse { error })
            })
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_revocation(
        &self,
        managing_id: &str,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<()> {
        let mut grant = self.get_managed_grant(managing_id).await?;
        let interaction = self.repo().recv_interaction().get_by_id(&grant.id).await?;
        self.gatekeeper().validate_managing_req(
            &grant,
            &interaction,
            "DELETE",
            &payload,
            &headers,
        )?;

        if grant.status == GrantStatus::Approved {
            grant.status = GrantStatus::Finalized;
            grant.ended_at = Some(Utc::now());
            grant.final_token_hash = None;
            grant.managing_token_hash = None;
            let grant = self.repo().recv_grant().update(grant).await?;
            self.gate_event(&grant, "revoked").await;
        }
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn sweep_expired(&self, now: DateTime<Utc>) -> Outcome<u64> {
        self.repo().recv_grant().finalize_expired(now).await
    }

    // ==========================================================================================
    // Internal steps of the flow
    // ==========================================================================================

    async fn gate_event(&self, grant: &grant::Model, action: &str) {
        let owner = recv_owner(grant);
        let payload = GrantEvent::from(grant);
        events::emit_action!(
            self.event_bus(),
            &owner,
            crate::EVENT_PREFIX,
            "gate",
            action,
            &payload
        );
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_managed_grant(&self, managing_id: &str) -> Outcome<grant::Model> {
        let grant = self
            .repo()
            .recv_grant()
            .get_by_managing_id(managing_id)
            .await?;
        if grant.kind != GrantKind::AccessToken {
            return Err(Errors::missing_resource(
                managing_id,
                "grant not found",
                None,
            ));
        }
        Ok(grant)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn inner_manage_rotation(
        &self,
        managing_id: &str,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let mut grant = self.get_managed_grant(managing_id).await?;
        let interaction = self.repo().recv_interaction().get_by_id(&grant.id).await?;
        self.gatekeeper().validate_managing_req(
            &grant,
            &interaction,
            "POST",
            &payload,
            &headers,
        )?;

        if grant.status != GrantStatus::Approved {
            return Err(Errors::security("Grant is not approved", None));
        }

        let now = Utc::now();
        let lifetime_reached = match grant.managing_expires_at {
            Some(at) => at <= now,
            None => true,
        };
        if lifetime_reached {
            grant.status = GrantStatus::Finalized;
            grant.ended_at = Some(now);
            let grant = self.repo().recv_grant().update(grant).await?;
            self.gate_event(&grant, "finalized").await;
            return Err(Errors::security(
                "Grant has reached its maximum lifetime",
                None,
            ));
        }

        let expected_managing_hash =
            require_field(grant.managing_token_hash.as_ref(), "managing token")?;
        let (rotation, issued) = self.gatekeeper().rotate_token(&grant, now)?;
        let mut rotated_grant = grant.clone();
        rotated_grant.final_token_hash = Some(rotation.final_token_hash.clone());
        rotated_grant.final_expires_at = Some(rotation.final_expires_at);
        rotated_grant.managing_token_hash = Some(rotation.managing_token_hash.clone());
        let rotated = self
            .repo()
            .recv_grant()
            .rotate_final(&grant.id, expected_managing_hash, rotation)
            .await?;
        if !rotated {
            return Err(Errors::security("Token was rotated concurrently", None));
        }
        self.gate_event(&rotated_grant, "rotated").await;

        let resource_req = self.repo().resource_req().get_by_id(&grant.id).await?;
        Ok(GrantResponse::token_issued(
            issued.final_token,
            &resource_req,
            issued.final_expires_in,
            issued.manage,
        ))
    }

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
        self.gate_event(&grant, "requested").await;
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
        let interaction = self
            .repo()
            .recv_interaction()
            .get_by_continuation_id(id)
            .await?;
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

        let issued = self.gatekeeper().issue_token(&mut grant, Utc::now());
        grant.participant_id = Some(holder.to_string());
        grant.status = GrantStatus::Approved;
        let grant = self.repo().recv_grant().update(grant).await?;
        self.gate_event(&grant, "approved").await;

        let resource_req = self
            .repo()
            .resource_req()
            .get_by_id(&interaction.id)
            .await?;
        Ok(GrantResponse::token_issued(
            issued.final_token,
            &resource_req,
            issued.final_expires_in,
            issued.manage,
        ))
    }
}
