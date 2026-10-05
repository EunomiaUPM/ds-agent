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

//! Parameter store.

pub mod service;
pub mod views;

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::version::Version;
use common::oauth::UserInfo;
use serde::Serialize;
use serde::de::DeserializeOwned;
use ymir::errors::Outcome;

/// Per-user parameters, the non-secret settings connectors and services read.
#[async_trait::async_trait]
pub trait ParameterStore<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Stores a new parameter of the caller.
    async fn create(&self, user: &UserInfo, cmd: &NewParameterCommand<T>) -> Outcome<Entry<T>>;

    /// 404 when the key does not exist for the caller.
    async fn read(&self, user: &UserInfo, key: &Key) -> Outcome<Entry<T>>;

    /// Replaces the value; fails when `expected_version` is stale.
    async fn update(
        &self,
        user: &UserInfo,
        key: &Key,
        cmd: &EditParameterCommand<T>,
        actor: &str,
    ) -> Outcome<Version>;

    async fn delete(&self, user: &UserInfo, key: &Key) -> Outcome<()>;

    /// Parameters under the filter's prefix.
    async fn list(&self, user: &UserInfo, filter: &PrefixFilter) -> Outcome<Vec<Entry<T>>>;

    /// Parameters found among `keys`; missing ones are left out.
    async fn batch(&self, user: &UserInfo, keys: &[Key]) -> Outcome<Vec<Entry<T>>>;

    async fn list_by_prefix(
        &self,
        user: &UserInfo,
        prefix: &KeyPrefix,
    ) -> Outcome<Vec<Entry<T>>> {
        self.list(user, &PrefixFilter::from(prefix)).await
    }
}
