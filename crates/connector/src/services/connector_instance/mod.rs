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

//! Connector instance use cases: resolve templates against parameters and persist.

pub(crate) mod service;

use crate::entities::connector_instance::{ConnectorInstanceDto, ConnectorInstantiationDto};
use common::auth::AccessScope;
use urn::Urn;
use ymir::errors::Outcome;

/// Service interface for connector instance operations.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ConnectorInstanceServiceTrait: Send + Sync {
    async fn get_instance_by_id(
        &self,
        scope: &AccessScope,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>>;
    async fn get_instance_by_distribution(
        &self,
        scope: &AccessScope,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>>;
    /// Validate parameters, resolve placeholders, and persist the instance.
    ///
    /// Idempotent: if an instance already exists for `distribution_id` it is
    /// updated in-place.
    async fn upsert_instance(
        &self,
        scope: &AccessScope,
        instance_dto: &mut ConnectorInstantiationDto,
    ) -> Outcome<ConnectorInstanceDto>;
    async fn delete_instance_by_id(&self, scope: &AccessScope, id: &Urn) -> Outcome<()>;
}
