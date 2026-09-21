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

pub mod service;
pub mod views;

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::version::Version;
use common::auth::AccessScope;
use serde::Serialize;
use serde::de::DeserializeOwned;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ParameterStore<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn create(&self, scope: &AccessScope, cmd: &NewParameterCommand<T>) -> Outcome<Entry<T>>;

    async fn read(&self, scope: &AccessScope, key: &Key) -> Outcome<Entry<T>>;

    async fn update(
        &self,
        scope: &AccessScope,
        key: &Key,
        cmd: &EditParameterCommand<T>,
        actor: &str,
    ) -> Outcome<Version>;

    async fn delete(&self, scope: &AccessScope, key: &Key) -> Outcome<()>;

    async fn list(&self, scope: &AccessScope, filter: &PrefixFilter) -> Outcome<Vec<Entry<T>>>;

    async fn batch(&self, scope: &AccessScope, keys: &[Key]) -> Outcome<Vec<Entry<T>>>;

    async fn list_by_prefix(
        &self,
        scope: &AccessScope,
        prefix: &KeyPrefix,
    ) -> Outcome<Vec<Entry<T>>> {
        self.list(scope, &PrefixFilter::from(prefix)).await
    }
}
