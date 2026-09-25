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

use async_trait::async_trait;
use common::auth::AccessScope;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use common::facades::VerifyTokenRequest;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

use crate::modules::ParticipantModule;

/// Peer token resolution without the HTTP hop; cross-tenant, like the tenant-less remote call.
pub struct SSIAuthLocalFacade {
    participants: Arc<dyn ParticipantModule>,
    home_tenant: String,
}

impl SSIAuthLocalFacade {
    /// `home_tenant` is the service client's tenant (`admin_seed.tenant_id`).
    pub fn new(participants: Arc<dyn ParticipantModule>, home_tenant: String) -> Self {
        Self {
            participants,
            home_tenant,
        }
    }
}

#[async_trait]
impl SSIAuthFacadeTrait for SSIAuthLocalFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn verify_token(&self, token: String) -> Outcome<Mates> {
        let scope = AccessScope::service_cross_tenant(&self.home_tenant);
        self.participants
            .get_by_token(&scope, VerifyTokenRequest { token })
            .await
    }
}
