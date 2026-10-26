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

//! Doctor over the closures and the profile registry; it reads, never writes.

use std::sync::Arc;

use common::oauth::OwnerScope;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::entities::resource_kind::ResourceKind;
use crate::services::doctor::views::DoctorReport;
use crate::services::doctor::DoctorServiceTrait;
use crate::setup::context::AppContext;

pub struct DoctorService {
    ctx: Arc<AppContext>,
}

impl DoctorService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl DoctorServiceTrait for DoctorService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn diagnose(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        profile: Option<String>,
        deep: bool,
    ) -> Outcome<DoctorReport> {
        let adapters = match &profile {
            Some(key) => vec![self.ctx.profiles.get(key).ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    format!(
                        "unknown profile {key}; available: {}",
                        self.ctx.profiles.keys().join(", ")
                    ),
                    None,
                )
            })?],
            None => self.ctx.profiles.all(),
        };
        let (root, set) = self
            .ctx
            .closures()
            .resource(&OwnerScope::seeing(user), id)
            .await?
            .filter(|(root, _)| root.kind() == kind)
            .ok_or_else(|| Errors::missing_resource(id.to_string(), format!("{} not found", kind.term()), None))?;
        let checked = if deep {
            set.ids()
        } else {
            vec![root.id().to_string()]
        };
        let profiles = adapters
            .iter()
            .map(|adapter| adapter.diagnose(&set, &root, deep))
            .collect::<Outcome<Vec<_>>>()?;
        Ok(DoctorReport {
            resource: root.id().to_string(),
            kind: kind.term().to_string(),
            checked,
            profiles,
        })
    }
}
