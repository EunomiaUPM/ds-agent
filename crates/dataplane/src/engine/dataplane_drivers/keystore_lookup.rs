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

use connector::KeystoreLookup;
use keystore::{Key, ParameterStore, SecretStore};

pub struct KeystoreClientImpl {
    parameter_store: Arc<dyn ParameterStore<serde_json::Value>>,
    secret_store: Arc<dyn SecretStore>,
}

impl KeystoreClientImpl {
    pub fn new(
        parameter_store: Arc<dyn ParameterStore<serde_json::Value>>,
        secret_store: Arc<dyn SecretStore>,
    ) -> Self {
        Self {
            parameter_store,
            secret_store,
        }
    }
}

impl KeystoreClientImpl {
    /// Lookups only read, and only the entries of `user_id` (the keystore is per user).
    fn reader_scope(user_id: &str) -> common::oauth::UserInfo {
        common::oauth::acting_as(user_id)
    }
}

#[async_trait::async_trait]
impl KeystoreLookup for KeystoreClientImpl {
    #[tracing::instrument(level = "info", skip_all, fields(peer.service = "keystore"))]
    async fn get_parameter(&self, user_id: &str, key: &str) -> Option<serde_json::Value> {
        let k = Key::new(key).ok()?;
        let user = Self::reader_scope(user_id);
        self.parameter_store
            .read(&user, &k)
            .await
            .ok()
            .map(|e| e.value)
    }

    #[tracing::instrument(level = "info", skip_all, fields(peer.service = "keystore"))]
    async fn get_secret(&self, user_id: &str, key: &str) -> Option<serde_json::Value> {
        let k = Key::new(key).ok()?;
        let user = Self::reader_scope(user_id);
        self.secret_store
            .read(&user, &k)
            .await
            .ok()
            .map(|e| e.value.expose().clone())
    }
}
