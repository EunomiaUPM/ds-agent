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

use std::sync::Arc;

use chrono::{DateTime, Utc};
use common::auth::AccessScope;
use common::auth::claims::Claims;
use common::paginated_spec::Cursor;
use common::query::QueryFilter;
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repositories::pat::PatRepository;
use crate::entities::pat::PersonalAccessToken;
use crate::entities::query::{Page, Paginated, PatFilter, Sort};
use crate::entities::role::RbacRole;
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::views::{CreatePatResponse, PatView};

pub struct PatService {
    pat_repo: Arc<dyn PatRepository>,
    event_bus: Option<events::EventBus>,
}

impl PatService {
    pub fn new(pat_repo: Arc<dyn PatRepository>) -> Self {
        Self {
            pat_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl PatServiceTrait for PatService {
    async fn create_pat(
        &self,
        scope: &AccessScope,
        name: &str,
        role: RbacRole,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Outcome<CreatePatResponse> {
        let target_tenant = scope.resolve_create_tenant(None)?;
        let (pat, raw_token) =
            PersonalAccessToken::generate(&target_tenant, name, role, scopes, expires_at);
        let created = self.pat_repo.create(&pat).await?;

        let res = CreatePatResponse {
            id: created.id,
            name: created.name,
            token: raw_token,
            token_prefix: created.token_prefix,
            role: created.role,
            scopes: created.scopes,
            expires_at: created.expires_at,
            created_at: created.created_at,
        };
        events::emit_action!(self.event_bus, crate::EVENT_PREFIX, "pat", "create", &res);
        Ok(res)
    }

    async fn list_pats(
        &self,
        scope: &AccessScope,
        filter: &PatFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PatView>> {
        scope.require_read()?;
        filter.validate()?;
        let mut filter = filter.clone();
        filter.tenant_id = scope.resolve_query_tenant(filter.tenant_id.as_deref())?;
        let page = page.clamped();
        let (pats, total) = tokio::try_join!(
            self.pat_repo.get_all(&filter, &page, sort),
            self.pat_repo.count(&filter),
        )?;
        let views: Vec<PatView> = pats.into_iter().map(PatView::assemble).collect();
        Ok(Paginated::from_page(views, &page, Some(total), |p| {
            Cursor::encode_timestamp(&p.created_at)
        }))
    }

    async fn revoke_pat(&self, scope: &AccessScope, id: Uuid) -> Outcome<()> {
        scope.require_write()?;
        self.pat_repo
            .revoke(scope.tenant_filter().map(str::to_string), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "pat",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }

    async fn validate_pat(&self, raw_token: &str) -> Outcome<Claims> {
        let hash = PersonalAccessToken::hash_token(raw_token);
        let pat = self
            .pat_repo
            .get_by_hash(&hash)
            .await?
            .ok_or_else(|| Errors::unauthorized("invalid or revoked PAT", None))?;

        if !pat.is_active() {
            return Err(Errors::unauthorized("PAT is expired or revoked", None));
        }

        let _ = self.pat_repo.update_last_used(pat.id).await;

        let now = Utc::now().timestamp();
        let exp = pat
            .expires_at
            .map(|dt| dt.timestamp())
            .unwrap_or(now + 31_536_000);

        Ok(Claims {
            sub: pat.tenant_id,
            role: pat.role,
            iat: pat.created_at.timestamp(),
            exp,
        })
    }
}
