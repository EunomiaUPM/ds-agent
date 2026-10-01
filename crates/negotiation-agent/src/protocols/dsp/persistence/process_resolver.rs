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

//! Resolves negotiation processes by DSP pid and authorizes the actor acting on them.

use crate::data::repo_traits::negotiation_process_repo::NegotiationProcessRepoTrait;
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use crate::services::negotiation_process::views::NegotiationProcessView;
use common::auth::{AccessScope, RbacRole};
use common::dsp_common::DspActor;
use common::errors::NotFoundExt;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

pub struct NegotiationProcessResolver {
    process_repo: Arc<dyn NegotiationProcessRepoTrait>,
    process_service: Arc<dyn NegotiationProcessServiceTrait>,
}

impl NegotiationProcessResolver {
    pub fn new(
        process_repo: Arc<dyn NegotiationProcessRepoTrait>,
        process_service: Arc<dyn NegotiationProcessServiceTrait>,
    ) -> Self {
        Self {
            process_repo,
            process_service,
        }
    }

    /// Scope the protocol acts under for a record: owner of the record's tenant, never global.
    pub fn owner_scope(tenant_id: &str) -> AccessScope {
        AccessScope::from_role(RbacRole::Owner, tenant_id)
    }

    /// Finds a process by one of its pids across tenants, since pids are global and the tenant is
    /// unknown until found, then checks that `actor` may act on it.
    #[tracing::instrument(level = "info", skip_all, err, fields(pid = %pid))]
    pub async fn resolve(&self, pid: &Urn, actor: &DspActor) -> Outcome<NegotiationProcessView> {
        let process = self
            .process_repo
            .get_negotiation_process_by_key_value(None, pid)
            .await?
            .or_not_found(pid, "negotiation process")?;
        actor.authorize(&process.tenant_id, &process.associated_agent_peer, pid)?;
        let id = Urn::from_str(&process.id)
            .map_err(|e| Errors::format(BadFormat::Received, e.to_string(), None))?;
        self.process_service
            .get_one(&Self::owner_scope(&process.tenant_id), &id)
            .await
    }
}
