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
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::version::Version;
use serde::Serialize;
use serde::de::DeserializeOwned;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ParameterStore<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn create(&self, cmd: &NewParameterCommand<T>) -> Outcome<Entry<T>>;

    async fn read(&self, key: &Key) -> Outcome<Entry<T>>;

    async fn update(
        &self,
        key: &Key,
        cmd: &EditParameterCommand<T>,
        actor: &str,
    ) -> Outcome<Version>;

    async fn delete(&self, key: &Key) -> Outcome<()>;

    async fn list(&self, prefix: &KeyPrefix) -> Outcome<Vec<Entry<T>>>;
}
