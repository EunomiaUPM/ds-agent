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

//! Secret store.

pub mod service;
pub mod views;

use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use common::auth::AccessScope;
use ymir::errors::Outcome;

/// Tenant-scoped secrets: credentials connectors and services resolve at runtime.
#[async_trait::async_trait]
pub trait SecretStore: Send + Sync {
    /// Stores a new secret in the tenant resolved from the scope.
    async fn create(&self, scope: &AccessScope, cmd: &NewSecretCommand) -> Outcome<SecretEntry>;

    /// 404 when the key does not exist for the caller.
    async fn read(&self, scope: &AccessScope, key: &Key) -> Outcome<SecretEntry>;

    /// Replaces the value; fails when `expected_version` is stale.
    async fn update(
        &self,
        scope: &AccessScope,
        key: &Key,
        cmd: &EditSecretCommand,
    ) -> Outcome<Version>;

    async fn delete(&self, scope: &AccessScope, key: &Key) -> Outcome<()>;

    /// Secrets under the filter's prefix.
    async fn list(&self, scope: &AccessScope, filter: &PrefixFilter) -> Outcome<Vec<SecretEntry>>;

    /// Secrets found among `keys`; missing ones are left out.
    async fn batch(&self, scope: &AccessScope, keys: &[Key]) -> Outcome<Vec<SecretEntry>>;

    /// Creates the secret or overwrites it whatever its version, in the acting tenant.
    async fn upsert(&self, scope: &AccessScope, key: &Key, value: SecretValue) -> Outcome<()>;

    async fn list_by_prefix(
        &self,
        scope: &AccessScope,
        prefix: &KeyPrefix,
    ) -> Outcome<Vec<SecretEntry>> {
        self.list(scope, &PrefixFilter::from(prefix)).await
    }
}
