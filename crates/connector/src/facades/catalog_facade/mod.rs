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

use ymir::errors::Outcome;

/// Served in-process by the catalog agent, which always hosts the connector.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait CatalogFacadeTrait: Send + Sync {
    /// Fails unless the catalog holds the distribution within `tenant_id`.
    async fn resolve_distribution_by_id(
        &self,
        tenant_id: &str,
        distribution_id: &str,
    ) -> Outcome<()>;
}
