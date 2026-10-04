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
use crate::types::response::TokenWhatResponse;
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
use ymir::types::listing::{GrantSort, SentGrantListFilter};
use ymir::types::oauth::UserInfo;
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
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
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
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
    async fn get_by_id(&self, user: &UserInfo, id: &str) -> Outcome<grant::Model> {
        let grant = self.get_access_grant(id).await?;
        user.ensure_sees(&grant.user_id, &grant.role, &grant.visibility, id)?;
        Ok(grant.seen_by(user))
    }

    /// The grant with its resource request, as `user` may get it. Its interaction and
    /// verification, which carry the GNAP continuation token, only for who reaches the grant.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
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
    // Flow entry points, called by the routers
    // ==========================================================================================

    /// Sends a grant request to the peer and follows its answer.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
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

    /// Answers the pending presentation request of grant `id` through the wallet, if `user`
    /// reaches the grant: acting needs more than seeing it (the verification shares its id).
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.user_id()))]
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
