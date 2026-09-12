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
use common::auth::claims::Claims;
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repositories::pat::PatRepository;
use crate::entities::pat::PersonalAccessToken;
use crate::entities::role::RbacRole;
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::views::{CreatePatResponse, PatView};

pub(crate) struct PatService {
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
        tenant_id: &str,
        name: &str,
        role: RbacRole,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Outcome<CreatePatResponse> {
        let (pat, raw_token) =
            PersonalAccessToken::generate(tenant_id, name, role, scopes, expires_at);
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

    async fn list_pats(&self, tenant_id: &str) -> Outcome<Vec<PatView>> {
        let pats = self.pat_repo.list_by_tenant(tenant_id).await?;
        Ok(pats.into_iter().map(PatView::assemble).collect())
    }

    async fn revoke_pat(&self, tenant_id: &str, id: Uuid) -> Outcome<()> {
        let pat = self
            .pat_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "PAT not found", None))?;

        if pat.tenant_id != tenant_id {
            return Err(Errors::format(
                BadFormat::Received,
                "unauthorized to revoke this PAT",
                None,
            ));
        }

        self.pat_repo.revoke(id).await?;
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
