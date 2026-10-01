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

use crate::entities::connector_instance::ConnectorInstanceDto;
use urn::Urn;
use ymir::errors::Outcome;

pub mod local;
pub mod remote;

/// Connector instances as seen by agents outside the catalog (dataplane, transfer).
#[mockall::automock]
#[async_trait::async_trait]
pub trait ConnectorInstanceFacadeTrait: Send + Sync {
    async fn get_instance_by_id(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>>;

    async fn get_instance_by_distribution(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>>;
}
