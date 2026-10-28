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
use crate::types::token_lifetimes::PENDING_GRANT_TTL_SECS;
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use chrono::{DateTime, Duration, Utc};
use common::facades::grants_facade::VerifiedPeer;
use common::oauth::OwnershipTrait;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::received::{grant, interaction};
use ymir::data::entities::shared::{registered_dataset, resource_req};
use ymir::errors::Errors;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::access_token::BoundToken;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::grant_request::access::{
    dsp_base, location_matches, AccessNeed, AccessType, ResourceAccess,
};
use ymir::types::gnap::grant_response::{Continuation, ErrorCode, ErrorResponse, GrantResponse};
use ymir::types::gnap::GrantStatus;
use ymir::types::listing::{GrantSort, RecvGrantListFilter};
use ymir::types::oauth::{RolePath, RoleTrait, UserInfo};
use ymir::types::vcs::VcType;
use ymir::types::verification::VerificationStatus;
use ymir::utils::{create_opaque_token, errors_to_error_code, hash_token, require_field};
use crate::types::token::IssuedToken;

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
            user_id: user.id().to_string(),
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

    /// The received grant `id` if `user` sees it, by its role or because it grants a dataset
    /// `user` sees; missing-resource error otherwise.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id(&self, user: &UserInfo, id: &str) -> Outcome<grant::Model> {
        let grant = self.repo().recv_grant().get_by_id(id).await?;
        let seen = user.ensure_sees_team(&grant.role, &grant.visibility, id);
        if seen.is_err() {
            let granted = self.repo().resource_req().get_by_id(id).await?.granted;
            let datasets = self.granted_datasets(granted.as_deref().unwrap_or_default()).await?;
            if !datasets.iter().any(|d| sees_dataset(user, d)) {
                seen?;
            }
        }
        Ok(grant)
    }

    /// The received grant with its resource request, interaction and verification, if `user`
    /// sees the grant; of what it grants, only the registered datasets `user` sees.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id_with_details(&self, user: &UserInfo, id: &str) -> Outcome<Value> {
        let grant = self.get_by_id(user, id).await?;
        let mut resource_req = self.repo().resource_req().get_by_id(id).await?;
        if let Some(granted) = resource_req.granted.take() {
            let hidden: Vec<String> = self
                .granted_datasets(&granted)
                .await?
                .into_iter()
                .filter(|d| !sees_dataset(user, d))
                .map(|d| d.id)
                .collect();
            resource_req.granted = Some(
                granted
                    .into_iter()
                    .filter(|e| e.identifier.as_ref().is_none_or(|id| !hidden.contains(id)))
                    .collect(),
            );
        }
        let interaction = self.repo().recv_interaction().get_latest_by_grant(id).await.ok();
        let verifications = self.repo().recv_verification().get_all_by_grant(id).await?;
        Ok(json!({
            "grant": grant,
            "resource_req": resource_req,
            "interaction": interaction,
            "verifications": verifications,
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

    /// The peer behind `token` if it also covers `need`; missing-resource error otherwise.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn authorize(&self, token: &str, need: &AccessNeed) -> Outcome<VerifiedPeer> {
        let grant = self
            .repo()
            .recv_grant()
            .get_valid_by_final_hash(&hash_token(token), Utc::now())
            .await?;
        let granted = self.repo().resource_req().get_by_id(&grant.id).await?.granted;
        if !need.covered_by(granted.as_deref().unwrap_or_default()) {
            return Err(Errors::missing_resource(&grant.id, "access not granted", None));
        }
        self.verify_token(token).await
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

    /// Modifies an approved grant (`PATCH`); failures become a GNAP error body.
    #[tracing::instrument(level = "info", skip_all)]
    async fn manage_modify_req(
        &self,
        id: &str,
        payload: Bytes,
        headers: HeaderMap,
    ) -> GrantResponse {
        self.inner_manage_modify_req(id, payload, headers)
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
        let interaction = self.repo().recv_interaction().get_latest_by_grant(&grant.id).await?;
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
        let expired = self.repo().recv_grant().finalize_expired(now).await?;
        let stale = self
            .repo()
            .recv_grant()
            .finalize_stale_pending(now - Duration::seconds(PENDING_GRANT_TTL_SECS), now)
            .await?;
        self.repo()
            .recv_verification()
            .expire_stale_pending(now - Duration::seconds(PENDING_GRANT_TTL_SECS), now)
            .await?;
        Ok(expired + stale)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn disconnect(&self, user: &UserInfo, id: &str) -> Outcome<()> {
        let mut grant = self.repo().recv_grant().get_by_id(id).await?;
        user.ensure_handles(&grant.role, id)?;
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
        let interaction = self.repo().recv_interaction().get_latest_by_grant(&grant.id).await?;
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
            None,
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

        let vcs = self.required_vcs(&resource_req.access).await?;
        let grant = self.repo().recv_grant().create(grant).await?;
        let interaction = self.repo().recv_interaction().create(interaction).await?;
        self.repo().resource_req().create(resource_req).await?;

        let verification = self.verifier().build_vp_plan(&grant.id, &vcs)?;
        let verification = self.repo().recv_verification().create(verification).await?;
        let uri = self.verifier().generate_verification_uri(&verification).await?;
        self.gate_event(&grant, "requested").await;
        Ok(GrantResponse::pending(uri, &interaction))
    }

    /// Checks the continuation against its interaction (the peer proves its key) and, with the
    /// presentation verified, approves the grant (or its modification) with a new token. On the
    /// first approval stores the peer (if new; an existing one is left as it is), the private
    /// relation of the verification with it, and which peer the grant belongs to.
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

        let mut grant = self.repo().recv_grant().get_by_id(&interaction.grant_id).await?;
        let verification = self
            .repo()
            .recv_verification()
            .get_latest_by_grant(&grant.id)
            .await?;
        if verification.status != VerificationStatus::Verified {
            return Err(Errors::security("The presentation is not verified", None));
        }
        let holder = require_field(verification.holder.as_ref(), "holder")?.to_string();
        let first = match grant.status {
            GrantStatus::Pending => true,
            GrantStatus::Approved if grant.participant_id.as_deref() == Some(holder.as_str()) => {
                false
            }
            _ => return Err(Errors::security("The grant cannot be continued", None)),
        };
        let resource_req = self.grant_requested(&mut grant).await?;

        let now = Utc::now();
        let (issued, action) = if first {
            let mate = self.gatekeeper().build_mate_plan(
                &holder,
                &grant.participant_nick,
                &interaction.callback_uri,
            );
            self.repo().participant().create_if_absent(mate).await?;
            let relation = self.gatekeeper().build_mate_rel_plan(&grant.role, &holder);
            self.repo().participant_relation().force_update(relation).await?;

            let issued = self.gatekeeper().issue_token(&mut grant, now);
            grant.participant_id = Some(holder);
            grant.status = GrantStatus::Approved;
            (issued, "approved")
        } else {
            (self.reissue_token(&mut grant, now)?, "modified")
        };
        let grant = self.repo().recv_grant().update(grant).await?;
        self.gate_event(&grant, action).await;

        let r#continue = self.rotate_continuation(interaction).await?;
        Ok(GrantResponse::token_issued(
            issued.final_token,
            &resource_req,
            issued.final_expires_in,
            issued.manage,
            Some(r#continue),
        ))
    }

    /// New access for an approved grant: a token at once, or a presentation if more is needed.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn inner_manage_modify_req(
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
        let modify = self
            .gatekeeper()
            .validate_modify_req(&interaction, &payload, &headers)?;

        let mut grant = self.repo().recv_grant().get_by_id(&interaction.grant_id).await?;
        if grant.status != GrantStatus::Approved {
            return Err(Errors::security("Only an approved grant can be modified", None));
        }

        let mut resource_req = self.repo().resource_req().get_by_id(&grant.id).await?;
        resource_req.access = modify.access_token.access;
        resource_req.flags = modify.access_token.flags;
        if modify.access_token.label.is_some() {
            resource_req.label = modify.access_token.label;
        }
        let resource_req = self.repo().resource_req().update(resource_req).await?;

        let required = self.required_vcs(&resource_req.access).await?;
        let missing = self.missing_credentials(&grant.id, required).await?;
        let covered = missing.is_empty();
        if !covered && modify.interact.is_some() {
            let round = self.gatekeeper().build_round_plan(&interaction, modify.interact)?;
            let round = self.repo().recv_interaction().create(round).await?;
            let verification = self.verifier().build_vp_plan(&grant.id, &missing)?;
            let verification = self.repo().recv_verification().create(verification).await?;
            let uri = self.verifier().generate_verification_uri(&verification).await?;
            self.gate_event(&grant, "modifying").await;
            return Ok(GrantResponse::pending(uri, &round));
        }

        let resource_req = match covered {
            true => self.grant_requested(&mut grant).await?,
            false => resource_req,
        };
        let issued = self.reissue_token(&mut grant, Utc::now())?;
        let grant = self.repo().recv_grant().update(grant).await?;
        self.gate_event(&grant, "modified").await;
        let r#continue = self.rotate_continuation(interaction).await?;
        Ok(GrantResponse::token_issued(
            issued.final_token,
            &resource_req,
            issued.final_expires_in,
            issued.manage,
            Some(r#continue),
        ))
    }

    /// New tokens for an approved grant, within its lifetime; the previous ones stop working.
    fn reissue_token(&self, grant: &mut grant::Model, now: DateTime<Utc>) -> Outcome<IssuedToken> {
        let (rotation, issued) = self.gatekeeper().rotate_token(grant, now)?;
        grant.final_token_hash = Some(rotation.final_token_hash);
        grant.final_expires_at = Some(rotation.final_expires_at);
        grant.managing_token_hash = Some(rotation.managing_token_hash);
        Ok(issued)
    }

    /// The registered datasets among those `granted` names by id.
    async fn granted_datasets(
        &self,
        granted: &[ResourceAccess],
    ) -> Outcome<Vec<registered_dataset::Model>> {
        let ids: Vec<String> = granted
            .iter()
            .filter(|e| e.r#type == AccessType::Dataset)
            .filter_map(|e| e.identifier.clone())
            .collect();
        match ids.is_empty() {
            true => Ok(Vec::new()),
            false => self.repo().registered_dataset().get_by_ids(&ids).await,
        }
    }

    /// Leaves exactly `datasets` registered, as the catalog publishes them.
    #[tracing::instrument(level = "info", skip_all, err, fields(datasets = datasets.len()))]
    async fn register_datasets(&self, datasets: Vec<registered_dataset::Model>) -> Outcome<()> {
        self.repo().registered_dataset().replace_all(datasets).await
    }

    /// The credentials `access` asks for: the base, plus what each registered dataset asked by id
    /// adds. By `locations` only the base: each dataset is then granted if covered.
    async fn required_vcs(&self, access: &[ResourceAccess]) -> Outcome<Vec<VcType>> {
        let ids: Vec<String> = access
            .iter()
            .filter(|e| e.r#type == AccessType::Dataset)
            .filter_map(|e| e.identifier.clone())
            .collect();
        let extras: Vec<VcType> = match ids.is_empty() {
            true => Vec::new(),
            false => self
                .repo()
                .registered_dataset()
                .get_by_ids(&ids)
                .await?
                .into_iter()
                .flat_map(|d| d.requirements)
                .collect(),
        };
        Ok(self.gatekeeper().required_vcs(&extras))
    }

    /// Grants what the verified credentials cover now (GNAP4DS 3.3), so a dataset published later
    /// needs a `PATCH`. A `dataset` by `locations` becomes the `catalog` of each location and one
    /// `dataset` per covered one registered there, and the grant passes to the longest role they
    /// all hang from; a `dataset` by id, if registered and covered; the rest as asked.
    async fn grant_requested(&self, grant: &mut grant::Model) -> Outcome<resource_req::Model> {
        let mut resource_req = self.repo().resource_req().get_by_id(&grant.id).await?;
        let shown = self.shown_credentials(&grant.id).await?;
        let covers = |d: &registered_dataset::Model| d.requirements.iter().all(|vc| shown.contains(vc));
        let is_dataset = |e: &ResourceAccess| e.r#type == AccessType::Dataset;
        let locations: Vec<String> = resource_req
            .access
            .iter()
            .filter(|e| is_dataset(*e) && e.identifier.is_none())
            .flat_map(|e| e.locations.clone().unwrap_or_default())
            .collect();
        let ids: Vec<String> = resource_req
            .access
            .iter()
            .filter(|e| is_dataset(*e))
            .filter_map(|e| e.identifier.clone())
            .collect();

        let registry = self.repo().registered_dataset();
        let under = |d: &registered_dataset::Model| {
            locations.iter().any(|l| location_matches(&d.location, l))
        };
        let by_location: Vec<_> = match locations.is_empty() {
            true => Vec::new(),
            false => registry
                .get_all()
                .await?
                .into_iter()
                .filter(|d| under(d) && covers(d))
                .collect(),
        };
        let by_id: Vec<_> = match ids.is_empty() {
            true => Vec::new(),
            false => registry.get_by_ids(&ids).await?.into_iter().filter(covers).collect(),
        };
        if !by_location.is_empty() {
            let roles: Vec<RolePath> = by_location.iter().map(|d| d.owner_role.clone()).collect();
            grant.role = common_root(&roles);
        }

        let mut granted: Vec<ResourceAccess> = Vec::new();
        let elements = resource_req
            .access
            .iter()
            .filter(|e| !is_dataset(*e))
            .cloned()
            .chain(locations.iter().map(|l| ResourceAccess::catalog(dsp_base(l))))
            .chain(by_location.iter().chain(&by_id).map(|d| ResourceAccess::dataset(d.id.clone())));
        for element in elements {
            if !granted.contains(&element) {
                granted.push(element);
            }
        }
        resource_req.granted = Some(granted);
        self.repo().resource_req().update(resource_req).await
    }

    /// The credentials the grant's verified presentations have shown.
    async fn shown_credentials(&self, grant_id: &str) -> Outcome<Vec<VcType>> {
        Ok(self
            .repo()
            .recv_verification()
            .get_all_by_grant(grant_id)
            .await?
            .into_iter()
            .filter(|v| v.status == VerificationStatus::Verified)
            .flat_map(|v| v.vc_type)
            .collect())
    }

    /// Of `required`, the credentials no verified presentation of the grant has shown.
    async fn missing_credentials(
        &self,
        grant_id: &str,
        required: Vec<VcType>,
    ) -> Outcome<Vec<VcType>> {
        let shown = self.shown_credentials(grant_id).await?;
        Ok(required
            .into_iter()
            .filter(|vc_type| !shown.contains(vc_type))
            .collect())
    }

    /// New continuation token; the old interaction reference stops working.
    async fn rotate_continuation(&self, mut interaction: interaction::Model) -> Outcome<Continuation> {
        interaction.continuation_token = create_opaque_token();
        interaction.interact_ref = create_opaque_token();
        let interaction = self.repo().recv_interaction().update(interaction).await?;
        Ok(Continuation {
            uri: interaction.continuation_endpoint,
            wait: None,
            access_token: BoundToken::new(interaction.continuation_token),
        })
    }
}

/// Whether `user` sees `dataset` inside the connector, by its owner.
fn sees_dataset(user: &UserInfo, dataset: &registered_dataset::Model) -> bool {
    user.sees_record(&dataset.owner_user_id, &dataset.owner_role, &dataset.owner_visibility)
}

/// The longest role path every one of `roles` is or hangs from; the root when they share none.
fn common_root(roles: &[RolePath]) -> RolePath {
    let segments = |role: &RolePath| role.as_str().split('/').skip(1).map(str::to_string).collect::<Vec<_>>();
    let Some((first, rest)) = roles.split_first() else {
        return RolePath::root();
    };
    let mut common = segments(first);
    for role in rest {
        let shared = common.iter().zip(segments(role)).take_while(|(a, b)| **a == *b).count();
        common.truncate(shared);
    }
    format!("/{}", common.join("/")).parse().unwrap_or_else(|_| RolePath::root())
}
