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
use common::auth::AccessScope;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::sent::{grant, verification};
use ymir::errors::Outcome;
use ymir::services::HasWallet;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus};
use ymir::types::listing::{GrantSort, SentGrantListFilter};
use ymir::types::verification::VerificationStatus;
use ymir::types::wallet::OidcUri;

#[async_trait]
pub trait PeerConnectorModule:
    HasPeerConnector + HasRepo + HasCallback + HasWallet + Send + Sync + 'static
{
    async fn req_peer_connection(
        &self,
        scope: &AccessScope,
        payload: ReachProvider,
    ) -> Outcome<()> {
        scope.require_write()?;
        let tenant_id = scope.acting_tenant();
        let actions = payload.actions.clone();
        let grant = self.peer_connector().build_grant_plan(tenant_id, payload);
        let interaction = self
            .peer_connector()
            .build_interaction_plan(tenant_id, &grant.id);
        let resource_req = self
            .peer_connector()
            .build_resource_req_plan(tenant_id, &grant.id, actions);

        let mut grant = self.repo().sent_grant().create(grant).await?;
        let mut interaction = self.repo().sent_interaction().create(interaction).await?;
        let resource_req = self.repo().resource_req().create(resource_req).await?;

        let grant_resp = self
            .peer_connector()
            .send_grant_req(&grant, &interaction, &resource_req)
            .await?;

        let what_response =
            self.peer_connector()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        self.manage_what_resp(grant, what_response).await
    }

    async fn manage_interaction_finish(&self, id: String, payload: CallbackBody) -> Outcome<()> {
        match payload {
            CallbackBody::Approved(payload) => self.req_peer_continuation(id, payload).await,
            CallbackBody::Rejected(_payload) => self.manage_rejection(id).await,
        }
    }

    // =================================== GETTERS FOR FRONTEND ====================================
    async fn get_all(
        &self,
        scope: &AccessScope,
        filter: &SentGrantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<grant::Model>> {
        let list_page = page.list_page(sort, GrantSort::Created, GrantSort::Updated)?;
        let list_filter = SentGrantListFilter {
            tenant_id: scope.tenant_filter().map(str::to_string),
            kind: filter.kind.clone().unwrap_or(GrantKind::AccessToken),
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

    async fn get_by_id(&self, scope: &AccessScope, id: String) -> Outcome<grant::Model> {
        let grant = self.repo().sent_grant().get_by_id(&id).await?;
        scope.ensure_visible(&grant.tenant_id, &id)?;
        Ok(grant)
    }

    async fn get_by_id_with_details(&self, scope: &AccessScope, id: String) -> Outcome<Value> {
        let grant = self.get_by_id(scope, id.clone()).await?;
        let resource_req = self.repo().resource_req().get_by_id(&id).await?;
        let interaction = self.repo().sent_interaction().get_by_id(&id).await.ok();
        let verification = self.repo().sent_verification().get_by_id(&id).await.ok();
        Ok(json!({
            "grant": grant,
            "resource_req": resource_req,
            "interaction": interaction,
            "verification": verification,
        }))
    }

    async fn process_oid4vp(
        &self,
        scope: &AccessScope,
        id: String,
        payload: OidcUri,
    ) -> Outcome<()> {
        scope.require_write()?;
        let mut verification = self.repo().sent_verification().get_by_id(&id).await?;
        scope.ensure_visible(&verification.tenant_id, &id)?;
        match self.wallet().process_oid4vp(&payload.uri).await {
            Ok(_) => verification.status = VerificationStatus::Verified,
            Err(_) => {
                verification.status = VerificationStatus::Failed;
            }
        }
        verification.ended_at = Some(Utc::now());
        self.repo().sent_verification().update(verification).await?;
        Ok(())
    }

    // ========================================= INTERNALS =========================================
    async fn manage_what_resp(
        &self,
        grant: grant::Model,
        vc_what_response: Outcome<TokenWhatResponse>,
    ) -> Outcome<()> {
        match vc_what_response? {
            TokenWhatResponse::Completed => {
                let mate = self.peer_connector().build_mate_plan(&grant);
                self.repo().participant().force_update(mate).await?;
                Ok(())
            }
            TokenWhatResponse::Presentation(uri) => self.manage_auto_oid4vp(&grant, &uri).await,
            TokenWhatResponse::Wait => Ok(()),
        }
    }

    async fn manage_oid4vp(&self, mut verification: verification::Model, uri: &str) -> Outcome<()> {
        match self.wallet().process_oid4vp(&uri).await {
            Ok(_) => verification.status = VerificationStatus::Verified,
            Err(_) => {
                verification.status = VerificationStatus::Failed;
            }
        }
        verification.ended_at = Some(Utc::now());
        self.repo().sent_verification().update(verification).await?;
        Ok(())
    }

    async fn manage_auto_oid4vp(&self, grant: &grant::Model, uri: &str) -> Outcome<()> {
        let verification =
            self.peer_connector()
                .build_verification_plan(&grant.tenant_id, uri, &grant.id)?;
        let verification = self.repo().sent_verification().create(verification).await?;

        if grant.auto {
            self.manage_oid4vp(verification, uri).await
        } else {
            Ok(())
        }
    }

    async fn req_peer_continuation(
        &self,
        id: String,
        payload: ApprovedCallbackBody,
    ) -> Outcome<()> {
        let mut interaction = self.repo().sent_interaction().get_by_id(&id).await?;
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        self.callback().apply_callback(&mut interaction, &payload);

        let result = self.callback().check_callback(&interaction, &grant);
        let mut interaction = self.repo().sent_interaction().update(interaction).await?;
        result?;

        let grant_resp = self.callback().send_continue_req(&interaction).await?;

        let what_response =
            self.peer_connector()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        self.manage_what_resp(grant, what_response).await
    }

    async fn manage_rejection(&self, id: String) -> Outcome<()> {
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        grant.status = GrantStatus::Rejected;
        grant.ended_at = Some(Utc::now());
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }
}
