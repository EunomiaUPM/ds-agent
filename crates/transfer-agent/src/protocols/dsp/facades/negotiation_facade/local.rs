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

use common::auth::AccessScope;
use negotiation_agent::AgreementView;
use negotiation_agent::services::agreement::AgreementServiceTrait;
use urn::Urn;
use ymir::errors::Outcome;

use crate::protocols::dsp::facades::negotiation_facade::NegotiationFacadeTrait;

/// Agreements read straight from the negotiation service, when it shares the process.
pub struct NegotiationLocalFacade {
    agreements: Arc<dyn AgreementServiceTrait>,
    service_tenant: String,
}

impl NegotiationLocalFacade {
    /// `service_tenant` is the service client's tenant (`admin_seed.tenant_id`).
    pub fn new(agreements: Arc<dyn AgreementServiceTrait>, service_tenant: String) -> Self {
        Self {
            agreements,
            service_tenant,
        }
    }
}

#[async_trait::async_trait]
impl NegotiationFacadeTrait for NegotiationLocalFacade {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "negotiation"))]
    async fn get_agreement(&self, agreement_id: &Urn) -> Outcome<AgreementView> {
        let scope = AccessScope::service_cross_tenant(&self.service_tenant);
        self.agreements.get_one(&scope, agreement_id).await
    }
}
