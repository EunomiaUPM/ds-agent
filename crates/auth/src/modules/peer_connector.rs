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

use crate::entities::filters::SentGrantFilter;
use crate::services::{HasCallback, HasPeerConnector, HasRepo};
use crate::types::entities::ReachProvider;
use crate::types::response::{RotationOutcome, TokenWhatResponse};
use crate::types::token_lifetimes::EXPIRY_MARGIN_SECS;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::errors::{Errors, Outcome};
use ymir::services::HasWallet;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus};
use ymir::types::listing::{GrantSort, SentGrantListFilter};
use ymir::types::oauth::{UserInfo, UserTrait};
use ymir::types::verification::VerificationStatus;
use ymir::types::wallet::OidcUri;

/// Onboarding with a peer through GNAP, presenting our credentials over OID4VP when asked.
#[async_trait]
pub trait PeerConnectorModule:
    HasPeerConnector + HasRepo + HasCallback + HasWallet + Send + Sync + 'static
{
    // ==========================================================================================
    // Sent grants: queries
    // ==========================================================================================

    /// Page of grants sent to peers, visible to the caller.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filter: &SentGrantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<grant::Model>> {
        let list_page = page.list_page(sort, GrantSort::Created, GrantSort::Updated)?;
        let list_filter = SentGrantListFilter {
            tenant: user.clone(),
            participant_id_contains: filter.participant_id.clone(),
            nick_contains: filter.participant_nick.clone(),
            status: filter.status.clone(),
            created_after: filter.created_after,
            created_before: filter.created_before,
        };
        let listed = self
            .repo()
            .sent_grant()
            .find_page(&list_filter, &list_page)
            .await?;
        let items = listed.items.into_iter().map(|g| g.seen_by(user)).collect();
        let sort_field = list_page.sort;
        Ok(Paginated::from_page(
            items,
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

    /// The access-token grant `id` as `user` may get it (see `grant::Model::seen_by`), if it
    /// sees the grant; missing-resource error otherwise.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id(&self, user: &UserInfo, id: &str) -> Outcome<grant::Model> {
        let grant = self.get_access_grant(id).await?;
        user.ensure_sees(&grant.user_id, &grant.role, &grant.visibility, id)?;
        Ok(grant.seen_by(user))
    }

    /// The grant with its resource request, as `user` may get it. Its interaction and
    /// verification, which carry the GNAP continuation token, only for who reaches the grant.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id_with_details(&self, user: &UserInfo, id: &str) -> Outcome<Value> {
        let grant = self.get_access_grant(id).await?;
        user.ensure_sees(&grant.user_id, &grant.role, &grant.visibility, id)?;
        let reaches = user.reaches(&grant.user_id, &grant.role);
        let resource_req = self.repo().resource_req().get_by_id(id).await?;
        let (interaction, verification) = if reaches {
            (
                self.repo().sent_interaction().get_by_id(id).await.ok(),
                self.repo().sent_verification().get_by_id(id).await.ok(),
            )
        } else {
            (None, None)
        };
        let grant = grant.seen_by(user);
        Ok(json!({
            "grant": grant,
            "resource_req": resource_req,
            "interaction": interaction,
            "verification": verification,
        }))
    }

    // ==========================================================================================
    // Sent grants: for the other agents (grants facade)
    // ==========================================================================================

    /// The token `user` presents to `participant_id`: the one of its own latest approved grant
    /// with that peer. `None` if it has none, even if a colleague does.
    ///
    /// A token about to expire is rotated, and a grant about to reach its lifetime is renewed with
    /// a new grant request. Obtaining the first grant with a peer (tokens plan, §4.4) is still up
    /// to the user.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn peer_token(&self, user: &UserInfo, participant_id: &str) -> Outcome<Option<String>> {
        let Some(grant) = self
            .repo()
            .sent_grant()
            .get_active_access(user.id(), participant_id)
            .await?
        else {
            return Ok(None);
        };

        let now = Utc::now();
        let margin = Duration::seconds(EXPIRY_MARGIN_SECS);
        let grant_ending = expires_within(grant.managing_expires_at, margin, now);
        let token_ending = expires_within(grant.final_expires_at, margin, now);

        if grant_ending || (grant.managing_uri.is_none() && token_ending) {
            return self.renew_access(user, participant_id, grant, now).await;
        }
        if token_ending {
            return self.rotate_access(user, participant_id, grant, now).await;
        }
        Ok(grant.final_token)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn sweep_expired(&self, now: DateTime<Utc>) -> Outcome<u64> {
        self.repo().sent_grant().finalize_expired(now).await
    }

    // ==========================================================================================
    // Flow entry points, called by the routers
    // ==========================================================================================

    /// Sends a grant request to the peer and follows its answer.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn req_peer_connection(&self, user: &UserInfo, payload: ReachProvider) -> Outcome<()> {
        let actions = payload.actions.clone();
        let grant = self.peer_connector().build_grant_plan(user, payload);
        let interaction = self.peer_connector().build_interaction_plan(&grant.id);
        let resource_req = self
            .peer_connector()
            .build_resource_req_plan(&grant.id, actions);

        let grant = self.repo().sent_grant().create(grant).await?;
        let interaction = self.repo().sent_interaction().create(interaction).await?;
        let resource_req = self.repo().resource_req().create(resource_req).await?;

        let grant_resp = self
            .peer_connector()
            .send_grant_req(&grant, &interaction, &resource_req)
            .await?;

        self.manage_what_resp(grant_resp, grant, interaction).await
    }

    /// Handles the peer's callback: continues on approval, marks the grant rejected otherwise.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_interaction_finish(&self, id: &str, payload: CallbackBody) -> Outcome<()> {
        match payload {
            CallbackBody::Approved(payload) => self.req_peer_continuation(id, payload).await,
            CallbackBody::Rejected(_payload) => self.manage_rejection(id).await,
        }
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn disconnect(&self, user: &UserInfo, id: &str) -> Outcome<()> {
        let grant = self.get_access_grant(id).await?;
        user.ensure_reaches(&grant.user_id, &grant.role, id)?;
        if grant.status == GrantStatus::Approved && grant.managing_uri.is_some() {
            if let Err(e) = self.peer_connector().send_revocation_req(&grant).await {
                e.log();
            }
        }
        self.finalize_sent(grant).await
    }

    /// Answers the pending presentation request of grant `id` through the wallet, if `user`
    /// reaches the grant: acting needs more than seeing it (the verification shares its id).
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn process_oid4vp(&self, user: &UserInfo, id: &str, payload: OidcUri) -> Outcome<()> {
        let grant = self.get_access_grant(id).await?;
        user.ensure_reaches(&grant.user_id, &grant.role, id)?;
        let verification = self.repo().sent_verification().get_by_id(id).await?;
        self.manage_oid4vp(verification, &payload.uri).await
    }

    // ==========================================================================================
    // Internal steps of the flow
    // ==========================================================================================

    /// The access-token grant `id`, whole; missing-resource error also for a VC request, which
    /// shares the table but is not served here. Visibility is up to the caller.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_access_grant(&self, id: &str) -> Outcome<grant::Model> {
        let grant = self.repo().sent_grant().get_by_id(id).await?;
        if grant.kind != GrantKind::AccessToken {
            return Err(Errors::missing_resource(id, "grant not found", None));
        }
        Ok(grant)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn rotate_access(
        &self,
        user: &UserInfo,
        participant_id: &str,
        grant: grant::Model,
        now: DateTime<Utc>,
    ) -> Outcome<Option<String>> {
        let usable = is_valid_at(grant.final_expires_at, now);
        let rotation = match self.peer_connector().send_rotation_req(&grant).await {
            Ok(response) => {
                let mut rotated = grant.clone();
                self.peer_connector()
                    .apply_rotation_resp(response, &mut rotated)
                    .map(|outcome| (outcome, rotated))
            }
            Err(e) => Err(e),
        };

        match rotation {
            Ok((RotationOutcome::Rotated, rotated)) => {
                let rotated = self.repo().sent_grant().update(rotated).await?;
                Ok(rotated.final_token)
            }
            Ok((RotationOutcome::Refused, _)) => {
                let current = self.repo().sent_grant().get_by_id(&grant.id).await?;
                if current.status == GrantStatus::Approved
                    && current.final_token != grant.final_token
                {
                    return Ok(current.final_token);
                }
                self.renew_access(user, participant_id, grant, now).await
            }
            Err(e) if usable => {
                e.log();
                Ok(grant.final_token)
            }
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn renew_access(
        &self,
        user: &UserInfo,
        participant_id: &str,
        grant: grant::Model,
        now: DateTime<Utc>,
    ) -> Outcome<Option<String>> {
        let processing_since = now - Duration::seconds(EXPIRY_MARGIN_SECS);
        let open = self
            .repo()
            .sent_grant()
            .has_open_access(user.id(), participant_id, processing_since)
            .await?;
        if !open {
            if let Err(e) = self.renew_grant(user, &grant).await {
                e.log();
            }
        }

        let latest = self
            .repo()
            .sent_grant()
            .get_active_access(user.id(), participant_id)
            .await?;
        if let Some(latest) = latest {
            if latest.id != grant.id && is_valid_at(latest.final_expires_at, now) {
                return Ok(latest.final_token);
            }
        }
        if is_valid_at(grant.final_expires_at, now) {
            return Ok(grant.final_token);
        }
        Ok(None)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn renew_grant(&self, user: &UserInfo, grant: &grant::Model) -> Outcome<()> {
        let resource_req = self.repo().resource_req().get_by_id(&grant.id).await?;
        let payload = ReachProvider {
            id: grant.participant_id.clone(),
            nick: grant.participant_nick.clone(),
            url: grant.grant_endpoint.clone(),
            actions: resource_req.actions,
            visibility: grant.visibility.clone(),
            auto: Some(grant.auto),
        };
        self.req_peer_connection(user, payload).await
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn finalize_sent(&self, mut grant: grant::Model) -> Outcome<()> {
        grant.status = GrantStatus::Finalized;
        grant.ended_at = Some(Utc::now());
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }

    /// Checks the callback, sends the GNAP continuation and follows its answer.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn req_peer_continuation(&self, id: &str, payload: ApprovedCallbackBody) -> Outcome<()> {
        let mut interaction = self.repo().sent_interaction().get_by_id(id).await?;
        let grant = self.repo().sent_grant().get_by_id(id).await?;
        self.callback().apply_callback(&mut interaction, &payload);

        let result = self.callback().check_callback(&interaction, &grant);
        let interaction = self.repo().sent_interaction().update(interaction).await?;
        result?;

        let grant_resp = self.callback().send_continue_req(&interaction).await?;

        self.manage_what_resp(grant_resp, grant, interaction).await
    }

    /// Applies the peer's answer to the grant and its interaction and stores both, even when
    /// the answer is an error. Then, once the grant completes, stores the peer (if new; an
    /// existing one is left as it is) and the relation of the user who sent the grant with it;
    /// otherwise starts the presentation the peer asks for, or waits.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_what_resp(
        &self,
        grant_resp: GrantResponse,
        mut grant: grant::Model,
        mut interaction: interaction::Model,
    ) -> Outcome<()> {
        let what_response =
            self.peer_connector()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);
        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        match what_response? {
            TokenWhatResponse::Completed => {
                let mate = self.peer_connector().build_mate_plan(&grant);
                self.repo().participant().create_if_absent(mate).await?;
                let relation = self.peer_connector().build_mate_relation(&grant);
                self.repo()
                    .participant_relation()
                    .force_update(relation)
                    .await?;
                Ok(())
            }
            TokenWhatResponse::Presentation(uri) => self.manage_auto_oid4vp(&grant, &uri).await,
            TokenWhatResponse::Wait => Ok(()),
        }
    }

    /// Records the presentation request; presents right away when the grant is automatic.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_auto_oid4vp(&self, grant: &grant::Model, uri: &str) -> Outcome<()> {
        let verification = self
            .peer_connector()
            .build_verification_plan(uri, &grant.id)?;
        let verification = self.repo().sent_verification().create(verification).await?;

        if grant.auto {
            self.manage_oid4vp(verification, uri).await
        } else {
            Ok(())
        }
    }

    /// Presents through the wallet and records whether it was verified.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_oid4vp(&self, mut verification: verification::Model, uri: &str) -> Outcome<()> {
        match self.wallet().process_oid4vp(uri).await {
            Ok(_) => verification.status = VerificationStatus::Verified,
            Err(_) => {
                verification.status = VerificationStatus::Failed;
            }
        }
        verification.ended_at = Some(Utc::now());
        self.repo().sent_verification().update(verification).await?;
        Ok(())
    }

    /// Marks grant `id` rejected, as the peer answered.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_rejection(&self, id: &str) -> Outcome<()> {
        let mut grant = self.repo().sent_grant().get_by_id(id).await?;
        grant.status = GrantStatus::Rejected;
        grant.ended_at = Some(Utc::now());
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }
}

fn expires_within(at: Option<DateTime<Utc>>, margin: Duration, now: DateTime<Utc>) -> bool {
    match at {
        Some(at) => at - margin <= now,
        None => false,
    }
}

fn is_valid_at(at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match at {
        Some(at) => at > now,
        None => true,
    }
}
