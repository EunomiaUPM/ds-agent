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
use urn::Urn;
use ymir::errors::Outcome;

use crate::entities::connector_instance::ConnectorInstanceDto;
use crate::facades::connector_instance_facade::ConnectorInstanceFacadeTrait;
use crate::services::connector_instance::ConnectorInstanceServiceTrait;

/// Instances read straight from the connector service, when it shares the process.
pub struct ConnectorInstanceLocalFacade {
    service: Arc<dyn ConnectorInstanceServiceTrait>,
}

impl ConnectorInstanceLocalFacade {
    pub fn new(service: Arc<dyn ConnectorInstanceServiceTrait>) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl ConnectorInstanceFacadeTrait for ConnectorInstanceLocalFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "connector", tenant = %tenant_id)
    )]
    async fn get_instance_by_id(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        self.service
            .get_instance_by_id(&AccessScope::service(tenant_id), id)
            .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "connector", tenant = %tenant_id)
    )]
    async fn get_instance_by_distribution(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        self.service
            .get_instance_by_distribution(&AccessScope::service(tenant_id), distribution_id)
            .await
    }
}
