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

//! Provider side of a transfer request: which connector instance serves the data an
//! agreement grants (agreement → dataset → distribution → connector instance).

use connector::ConnectorInstanceDto;
use urn::Urn;
use ymir::errors::Outcome;

pub mod connector_resolver;

#[async_trait::async_trait]
pub trait ConnectorResolverTrait: Send + Sync {
    async fn resolve_connector_by_agreement_id(
        &self,
        agreement_id: &Urn,
        formats: Option<&String>,
    ) -> Outcome<ConnectorInstanceDto>;
}
