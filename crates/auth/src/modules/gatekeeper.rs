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
use common::auth::AccessScope;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use serde_json::{json, Value};
use ymir::data::entities::received::grant;
use ymir::errors::{Errors, Outcome};
use ymir::services::HasVerifier;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::grant_response::{ErrorResponse, GrantResponse};
use ymir::types::gnap::GrantStatus;
use ymir::types::listing::{GrantSort, RecvGrantListFilter};
use ymir::utils::{create_opaque_token, errors_to_error_code, require_field};

#[async_trait]
pub trait GateKeeperModule: HasGateKeeper + HasVerifier + HasRepo + Send + Sync + 'static {
    #[tracing::instrument(level = "info", skip_all)]
    async fn manage_grant_req(
        &self,
        tenant_id: String,
        payload: Bytes,
        headers: HeaderMap,
    ) -> GrantResponse {
        self.inner_manage_grant_req(tenant_id, payload, headers)
            .await
            .unwrap_or_else(|e| {
                e.log();
                let code = errors_to_error_code(&e);
                GrantResponse::Error(ErrorResponse { error: code })
            })
    }

    #[tracing::instrument(level = "info", skip_all)]
    async fn manage_continue_req(
        &self,
        tenant_id: String,
        id: String,
        payload: Bytes,
        headers: HeaderMap,
    ) -> GrantResponse {
        self.inner_manage_continue_req(tenant_id, id, payload, headers)
            .await
            .unwrap_or_else(|e| {
                e.log();
                let code = errors_to_error_code(&e);
                GrantResponse::Error(ErrorResponse { error: code })
            })
    }

    // =================================== GETTERS FOR FRONTEND ====================================
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filter: &RecvGrantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<grant::Model>> {
        let list_page = page.list_page(sort, GrantSort::Created, GrantSort::Updated)?;
        let list_filter = RecvGrantListFilter {
            tenant_id: scope.tenant_filter().map(str::to_string),
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

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_by_id(&self, scope: &AccessScope, id: String) -> Outcome<grant::Model> {
        let grant = self.repo().recv_grant().get_by_id(&id).await?;
        scope.ensure_visible(&grant.tenant_id, &id)?;
        Ok(grant)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_by_id_with_details(&self, scope: &AccessScope, id: String) -> Outcome<Value> {
        let grant = self.get_by_id(scope, id.clone()).await?;
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

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn inner_manage_grant_req(
        &self,
        tenant_id: String,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let grant_request = self
            .gatekeeper()
            .validate_grant_req(&tenant_id, &payload, &headers)?;

        let grant = self
            .gatekeeper()
            .build_grant_plan(&tenant_id, grant_request.client.class_id.clone())?;
        let interaction = self.gatekeeper().build_interaction_plan(
            &tenant_id,
            &grant.id,
            grant_request.client,
            grant_request.interact,
        )?;
        let resource_req =
            self.gatekeeper()
                .build_resource_req_plan(&tenant_id, &grant.id, grant_request.kind)?;

        let grant = self.repo().recv_grant().create(grant).await?;
        let interaction = self.repo().recv_interaction().create(interaction).await?;
        let _resource_req = self.repo().resource_req().create(resource_req).await?;

        let verification = self.verifier().build_vp_plan(&grant.tenant_id, &grant.id)?;
        let ver_model = self.repo().recv_verification().create(verification).await?;
        let uri = self.verifier().generate_verification_uri(&ver_model);
        Ok(GrantResponse::pending(uri, &interaction))
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn inner_manage_continue_req(
        &self,
        tenant_id: String,
        id: String,
        payload: Bytes,
        headers: HeaderMap,
    ) -> Outcome<GrantResponse> {
        let interaction = self.repo().recv_interaction().get_by_cont_id(&id).await?;
        if interaction.tenant_id != tenant_id {
            return Err(Errors::missing_resource(id, "interaction not found", None));
        }
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
            &grant.tenant_id,
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
