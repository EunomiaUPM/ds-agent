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
use crate::utils::pagination::AuthPagination;
use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use common::paginated_spec::{Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::received::grant;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::grant_request::GrantKind;
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

    // =================================== GETTERS FOR FRONTEND ====================================
    async fn get_all(
        &self,
        filter: &RecvGrantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<grant::Model>> {
        let kind = filter.kind.clone().unwrap_or(GrantKind::AccessToken);
        let grants = self.repo().recv_grant().filter_by_type(kind).await?;

        let mut filtered: Vec<grant::Model> = grants
            .into_iter()
            .filter(|g| {
                if let Some(nick) = &filter.participant_nick {
                    if !g
                        .participant_nick
                        .to_lowercase()
                        .contains(&nick.to_lowercase())
                    {
                        return false;
                    }
                }
                if let Some(status) = &filter.status {
                    if &g.status != status {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if g.created_at < after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if g.created_at > before {
                        return false;
                    }
                }
                true
            })
            .collect();

        match sort {
            Sort::CreatedAtAsc => {
                filtered.sort_by(|a, b| {
                    a.created_at
                        .cmp(&b.created_at)
                        .then_with(|| a.id.cmp(&b.id))
                });
            }
            Sort::UpdatedAtAsc => {
                filtered.sort_by(|a, b| a.ended_at.cmp(&b.ended_at).then_with(|| a.id.cmp(&b.id)));
            }
            Sort::UpdatedAtDesc => {
                filtered.sort_by(|a, b| b.ended_at.cmp(&a.ended_at).then_with(|| a.id.cmp(&b.id)));
            }
            _ => {
                filtered.sort_by(|a, b| {
                    b.created_at
                        .cmp(&a.created_at)
                        .then_with(|| a.id.cmp(&b.id))
                });
            }
        }

        Ok(AuthPagination::paginate(
            &filtered,
            page,
            sort,
            |item, s| {
                let ts = match s {
                    Sort::UpdatedAtAsc | Sort::UpdatedAtDesc => {
                        item.ended_at.unwrap_or(item.created_at)
                    }
                    _ => item.created_at,
                };
                (ts, item.id.clone())
            },
        ))
    }

    async fn get_by_id(&self, id: String) -> Outcome<grant::Model> {
        self.repo().recv_grant().get_by_id(&id).await
    }

    async fn get_by_id_with_details(&self, id: String) -> Outcome<Value> {
        let grant = self.repo().recv_grant().get_by_id(&id).await?;
        let resource_req = self.repo().resource_req().get_by_id(&id).await?;
        let interaction = self.repo().recv_interaction().get_by_id(&id).await.ok();
        let verification = self.repo().recv_verification().get_by_id(&id).await.ok();
        Ok(json!({
            "grant": grant,
            "resource_req": resource_req,
            "interaction": interaction,
            "verification": verification,
        }))
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
