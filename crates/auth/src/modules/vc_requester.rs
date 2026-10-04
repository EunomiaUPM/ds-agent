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
use crate::services::{HasCallback, HasRepo, HasVcRequester};
use crate::types::entities::ReachAuthority;
use crate::types::response::VcWhatResponse;
use async_trait::async_trait;
use chrono::Utc;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::errors::{Errors, Outcome};
use ymir::services::HasWallet;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus};
use ymir::types::listing::{GrantSort, VcRequestListFilter};
use ymir::types::oauth::UserInfo;
use ymir::types::verification::VerificationStatus;
use ymir::types::wallet::OidcUri;

/// Requesting credentials from an authority through GNAP, then OID4VCI or OID4VP.
///
/// Credentials belong to the whole connector, so VC requests are created public: every user sees
/// them, but only who reaches one (its author, a role above, the root) gets its secrets and acts
/// on it.
#[async_trait]
pub trait VcRequesterModule:
    HasVcRequester + HasRepo + HasCallback + HasWallet + Send + Sync + 'static
{
    // ==========================================================================================
    // VC requests: queries
    // ==========================================================================================

    /// Page of the VC requests `user` sees, each as it may get it; they are created public, so in
    /// practice all of them.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filter: &SentGrantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<grant::Model>> {
        let list_page = page.list_page(sort, GrantSort::Created, GrantSort::Updated)?;
        let list_filter = VcRequestListFilter {
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
            .find_vc_requests_page(&list_filter, &list_page)
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

    /// The VC request `id` as `user` may get it (see `grant::Model::seen_by`), if it sees the
    /// request; missing-resource error otherwise.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn get_by_id(&self, user: &UserInfo, id: &str) -> Outcome<grant::Model> {
        let grant = self.get_vc_request(id).await?;
        user.ensure_sees(&grant.user_id, &grant.role, &grant.visibility, id)?;
        Ok(grant.seen_by(user))
    }

    /// The VC request as `user` may get it. Its interaction and verification, which carry the
    /// GNAP continuation token, only for who reaches the request.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn get_by_id_with_details(&self, user: &UserInfo, id: &str) -> Outcome<Value> {
        let grant = self.get_vc_request(id).await?;
        user.ensure_sees(&grant.user_id, &grant.role, &grant.visibility, id)?;
        let (interaction, verification) = if user.reaches(&grant.user_id, &grant.role) {
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
            "interaction": interaction,
            "verification": verification,
        }))
    }

    // ==========================================================================================
    // Flow entry points, called by the routers
    // ==========================================================================================

    /// Sends a grant request to the authority and follows its answer.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn beg_vc(&self, user: &UserInfo, payload: ReachAuthority) -> Outcome<()> {
        let start = payload.method.clone();
        let grant = self.vc_requester().build_grant_plan(user, payload);
        let interaction = self.vc_requester().build_interaction_plan(&grant.id, start);

        let grant = self.repo().sent_grant().create(grant).await?;
        let interaction = self.repo().sent_interaction().create(interaction).await?;

        let grant_resp = self
            .vc_requester()
            .send_grant_req(&grant, &interaction)
            .await?;

        self.manage_what_resp(grant_resp, grant, interaction).await
    }

    /// Handles the authority's callback: continues on approval, marks the request rejected
    /// otherwise.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_interaction_finish(&self, id: &str, payload: CallbackBody) -> Outcome<()> {
        match payload {
            CallbackBody::Approved(payload) => self.req_vc_continuation(id, payload).await,
            CallbackBody::Rejected(_payload) => self.manage_rejection(id).await,
        }
    }

    /// Accepts the credential offered for request `id` through the wallet and registers the
    /// authority, if `user` reaches the request: acting needs more than seeing it.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn process_oid4vci(&self, user: &UserInfo, id: &str, payload: OidcUri) -> Outcome<()> {
        let grant = self.get_vc_request(id).await?;
        user.ensure_reaches(&grant.user_id, &grant.role, id)?;
        self.wallet().process_oid4vci(&payload.uri).await?;
        self.finalize_issuance(grant).await
    }

    /// Answers the pending presentation request of request `id` through the wallet, if `user`
    /// reaches the request (the verification shares its id).
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn process_oid4vp(&self, user: &UserInfo, id: &str, payload: OidcUri) -> Outcome<()> {
        let grant = self.get_vc_request(id).await?;
        user.ensure_reaches(&grant.user_id, &grant.role, id)?;
        let verification = self.repo().sent_verification().get_by_id(id).await?;
        self.manage_oid4vp(verification, &payload.uri).await
    }

    // ==========================================================================================
    // Internal steps of the flow
    // ==========================================================================================

    /// The VC request `id`, whole; missing-resource error also for an access-token grant, which
    /// shares the table but is not served here. Visibility is up to the caller.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_vc_request(&self, id: &str) -> Outcome<grant::Model> {
        let grant = self.repo().sent_grant().get_by_id(id).await?;
        if grant.kind != GrantKind::CredentialRequest {
            return Err(Errors::missing_resource(id, "VC request not found", None));
        }
        Ok(grant)
    }

    /// Checks the callback, sends the GNAP continuation and follows its answer.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn req_vc_continuation(&self, id: &str, payload: ApprovedCallbackBody) -> Outcome<()> {
        let mut interaction = self.repo().sent_interaction().get_by_id(id).await?;
        let grant = self.repo().sent_grant().get_by_id(id).await?;
        self.callback().apply_callback(&mut interaction, &payload);

        let result = self.callback().check_callback(&interaction, &grant);
        let interaction = self.repo().sent_interaction().update(interaction).await?;
        result?;

        let grant_resp = self.callback().send_continue_req(&interaction).await?;

        self.manage_what_resp(grant_resp, grant, interaction).await
    }

    /// Applies the authority's answer to the request and its interaction and stores both, even
    /// when the answer is an error. Then starts the issuance or the presentation it asks for,
    /// or waits.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_what_resp(
        &self,
        grant_resp: GrantResponse,
        mut grant: grant::Model,
        mut interaction: interaction::Model,
    ) -> Outcome<()> {
        let what_response =
            self.vc_requester()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);
        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        match what_response? {
            VcWhatResponse::Issuance(uri) => self.manage_oid4vci(grant, &uri).await,
            VcWhatResponse::Presentation(uri) => self.manage_auto_oid4vp(&grant, &uri).await,
            VcWhatResponse::Wait => Ok(()),
        }
    }

    /// Accepts the credential right away when the request is automatic.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_oid4vci(&self, grant: grant::Model, uri: &str) -> Outcome<()> {
        if !grant.auto {
            return Ok(());
        }
        self.wallet().process_oid4vci(uri).await?;
        self.finalize_issuance(grant).await
    }

    /// Marks the request finalized once its credential is in the wallet, and stores the
    /// authority (if new; an existing one is left as it is) and the relation of the user who
    /// asked for it.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn finalize_issuance(&self, mut grant: grant::Model) -> Outcome<()> {
        grant.status = GrantStatus::Finalized;
        grant.ended_at = Some(Utc::now());
        let grant = self.repo().sent_grant().update(grant).await?;
        let authority = self.vc_requester().build_authority_plan(&grant);
        self.repo()
            .participant()
            .create_if_absent(authority)
            .await?;
        let relation = self.vc_requester().build_auth_relation(&grant);
        self.repo()
            .participant_relation()
            .force_update(relation)
            .await?;
        Ok(())
    }

    /// Records the presentation request; presents right away when the request is automatic.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_auto_oid4vp(&self, grant: &grant::Model, uri: &str) -> Outcome<()> {
        let verification = self
            .vc_requester()
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

    /// Marks request `id` rejected, as the authority answered.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn manage_rejection(&self, id: &str) -> Outcome<()> {
        let mut grant = self.repo().sent_grant().get_by_id(id).await?;
        grant.status = GrantStatus::Rejected;
        grant.ended_at = Some(Utc::now());
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }
}
