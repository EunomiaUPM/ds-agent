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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use common::paginated_spec::Cursor;
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::auth_code::AuthCodeRepository;
use crate::data::repositories::client::{ClientRepository, ClientRepositoryError};
use crate::data::repositories::pat::{PatRepository, PatRepositoryError};
use crate::data::repositories::token::TokenRepository;
use crate::data::repositories::user::{UserRepository, UserRepositoryError};
use crate::entities::auth_code::AuthCode;
use crate::entities::client::Client;
use crate::entities::pat::PersonalAccessToken;
use crate::entities::query::{ClientFilter, Page, PatFilter, Sort, UserFilter};
use crate::entities::refresh_token::RefreshToken;
use crate::entities::role::RbacRole;
use crate::entities::user::User;

// User ──────────────────────────────────────────────────────────────────────

pub(crate) struct InMemoryUserRepository {
    store: Arc<Mutex<HashMap<String, User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn get_all(&self, filter: &UserFilter, page: &Page, sort: &Sort) -> Outcome<Vec<User>> {
        let store = self.store.lock().unwrap();

        let cursor_dt = page
            .cursor
            .as_deref()
            .and_then(|c| Cursor::decode_utc_timestamp(c).ok());

        let mut users: Vec<User> = store
            .values()
            .filter(|u| {
                if let Some(ref tenant_id) = filter.tenant_id {
                    if u.tenant_id != *tenant_id {
                        return false;
                    }
                }
                if let Some(role) = filter.role {
                    if u.role != role {
                        return false;
                    }
                }
                if let Some(ref email) = filter.email {
                    if !u.email.contains(email.as_str()) {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if u.created_at <= after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if u.created_at >= before {
                        return false;
                    }
                }
                if let Some(cursor) = cursor_dt {
                    match sort {
                        Sort::CreatedAtAsc => {
                            if u.created_at <= cursor {
                                return false;
                            }
                        }
                        _ => {
                            if u.created_at >= cursor {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        match sort {
            Sort::CreatedAtAsc => users.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            _ => users.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        users.truncate(page.limit as usize);
        Ok(users)
    }

    async fn count(&self, filter: &UserFilter) -> Outcome<u64> {
        let store = self.store.lock().unwrap();
        let count = store
            .values()
            .filter(|u| {
                if let Some(ref tenant_id) = filter.tenant_id {
                    if u.tenant_id != *tenant_id {
                        return false;
                    }
                }
                if let Some(role) = filter.role {
                    if u.role != role {
                        return false;
                    }
                }
                if let Some(ref email) = filter.email {
                    if !u.email.contains(email.as_str()) {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if u.created_at <= after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if u.created_at >= before {
                        return false;
                    }
                }
                true
            })
            .count();
        Ok(count as u64)
    }

    async fn get_by_tenant_id(&self, tenant_id: &str) -> Outcome<Option<User>> {
        Ok(self.store.lock().unwrap().get(tenant_id).cloned())
    }

    async fn get_by_email(&self, email: &str) -> Outcome<Option<User>> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .find(|u| u.email == email)
            .cloned())
    }

    async fn create(&self, user: &User) -> Outcome<User> {
        let mut store = self.store.lock().unwrap();
        if store.contains_key(&user.tenant_id) {
            return Err(UserRepositoryError::AlreadyExists.into_errors());
        }
        store.insert(user.tenant_id.clone(), user.clone());
        Ok(user.clone())
    }

    async fn patch(
        &self,
        tenant_id: &str,
        email: Option<String>,
        role: Option<RbacRole>,
        extra_fields: Option<serde_json::Value>,
    ) -> Outcome<User> {
        let mut store = self.store.lock().unwrap();
        let user = store
            .get_mut(tenant_id)
            .ok_or_else(|| UserRepositoryError::NotFound.into_errors())?;
        if let Some(e) = email {
            user.email = e;
        }
        if let Some(r) = role {
            user.role = r;
        }
        if let Some(f) = extra_fields {
            user.extra_fields = f;
        }
        Ok(user.clone())
    }

    async fn delete(&self, tenant_id: &str) -> Outcome<()> {
        let mut store = self.store.lock().unwrap();
        if store.remove(tenant_id).is_none() {
            return Err(UserRepositoryError::NotFound.into_errors());
        }
        Ok(())
    }
}

// RefreshToken ──────────────────────────────────────────────────────────────

pub(crate) struct InMemoryRefreshTokenRepository {
    store: Arc<Mutex<HashMap<Uuid, RefreshToken>>>,
}

impl InMemoryRefreshTokenRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl TokenRepository for InMemoryRefreshTokenRepository {
    async fn create(&self, token: &RefreshToken) -> Outcome<RefreshToken> {
        self.store.lock().unwrap().insert(token.id, token.clone());
        Ok(token.clone())
    }

    async fn get_by_jti(&self, jti: &str) -> Outcome<Option<RefreshToken>> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .find(|t| t.jti == jti)
            .cloned())
    }

    async fn revoke(&self, id: Uuid) -> Outcome<()> {
        if let Some(t) = self.store.lock().unwrap().get_mut(&id) {
            t.revoked = true;
        }
        Ok(())
    }

    async fn revoke_all_for_tenant(&self, tenant_id: &str) -> Outcome<()> {
        self.store
            .lock()
            .unwrap()
            .values_mut()
            .filter(|t| t.tenant_id == tenant_id)
            .for_each(|t| {
                t.revoked = true;
            });
        Ok(())
    }
}

// Client ────────────────────────────────────────────────────────────────────

pub(crate) struct InMemoryClientRepository {
    store: Arc<Mutex<HashMap<String, Client>>>,
}

impl InMemoryClientRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl ClientRepository for InMemoryClientRepository {
    async fn get_all(
        &self,
        filter: &ClientFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<Client>> {
        let store = self.store.lock().unwrap();

        let cursor_dt = page
            .cursor
            .as_deref()
            .and_then(|c| Cursor::decode_utc_timestamp(c).ok());

        let mut clients: Vec<Client> = store
            .values()
            .filter(|c| {
                if let Some(ref tid) = filter.tenant_id {
                    if c.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(role) = filter.role {
                    if c.role != role {
                        return false;
                    }
                }
                if let Some(ref search) = filter.search {
                    let term = search.to_lowercase();
                    if !c.client_name.to_lowercase().contains(&term)
                        && !c.client_id.to_lowercase().contains(&term)
                    {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if c.created_at <= after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if c.created_at >= before {
                        return false;
                    }
                }
                if let Some(cursor) = cursor_dt {
                    match sort {
                        Sort::CreatedAtAsc => {
                            if c.created_at <= cursor {
                                return false;
                            }
                        }
                        _ => {
                            if c.created_at >= cursor {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        match sort {
            Sort::CreatedAtAsc => clients.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            _ => clients.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        clients.truncate(page.limit as usize);
        Ok(clients)
    }

    async fn count(&self, filter: &ClientFilter) -> Outcome<u64> {
        let store = self.store.lock().unwrap();
        let count = store
            .values()
            .filter(|c| {
                if let Some(ref tid) = filter.tenant_id {
                    if c.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(role) = filter.role {
                    if c.role != role {
                        return false;
                    }
                }
                if let Some(ref search) = filter.search {
                    let term = search.to_lowercase();
                    if !c.client_name.to_lowercase().contains(&term)
                        && !c.client_id.to_lowercase().contains(&term)
                    {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if c.created_at <= after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if c.created_at >= before {
                        return false;
                    }
                }
                true
            })
            .count();
        Ok(count as u64)
    }

    async fn get_by_id(&self, tenant_id: &str, client_id: &str) -> Outcome<Option<Client>> {
        let store = self.store.lock().unwrap();
        let item = store.get(client_id).cloned();
        Ok(item.filter(|c| c.tenant_id == tenant_id))
    }

    async fn get_batch(&self, tenant_id: &str, client_ids: &[String]) -> Outcome<Vec<Client>> {
        let store = self.store.lock().unwrap();
        let mut result = Vec::new();
        for id in client_ids {
            if let Some(client) = store.get(id) {
                if client.tenant_id == tenant_id {
                    result.push(client.clone());
                }
            }
        }
        Ok(result)
    }

    async fn get_by_client_id(&self, client_id: &str) -> Outcome<Option<Client>> {
        Ok(self.store.lock().unwrap().get(client_id).cloned())
    }

    async fn create(&self, client: &Client) -> Outcome<Client> {
        let mut store = self.store.lock().unwrap();
        if store.contains_key(&client.client_id) {
            return Err(ClientRepositoryError::AlreadyExists.into_errors());
        }
        store.insert(client.client_id.clone(), client.clone());
        Ok(client.clone())
    }

    async fn delete(&self, tenant_id: &str, client_id: &str) -> Outcome<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(c) = store.get(client_id) {
            if c.tenant_id != tenant_id {
                return Err(ClientRepositoryError::NotFound.into_errors());
            }
        } else {
            return Err(ClientRepositoryError::NotFound.into_errors());
        }
        store.remove(client_id);
        Ok(())
    }
}

// AuthCode ──────────────────────────────────────────────────────────────────

pub(crate) struct InMemoryAuthCodeRepository {
    store: Arc<Mutex<HashMap<String, AuthCode>>>,
}

impl InMemoryAuthCodeRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl AuthCodeRepository for InMemoryAuthCodeRepository {
    async fn save(&self, auth_code: &AuthCode) -> Outcome<AuthCode> {
        let mut store = self.store.lock().unwrap();
        store.insert(auth_code.code.clone(), auth_code.clone());
        Ok(auth_code.clone())
    }

    async fn get_by_code(&self, code: &str) -> Outcome<Option<AuthCode>> {
        Ok(self.store.lock().unwrap().get(code).cloned())
    }

    async fn mark_used(&self, code: &str) -> Outcome<()> {
        if let Some(entry) = self.store.lock().unwrap().get_mut(code) {
            entry.used = true;
        }
        Ok(())
    }
}

// Personal Access Token ─────────────────────────────────────────────────────

pub(crate) struct InMemoryPatRepository {
    store: Arc<Mutex<HashMap<Uuid, PersonalAccessToken>>>,
}

impl InMemoryPatRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl PatRepository for InMemoryPatRepository {
    async fn get_all(
        &self,
        filter: &PatFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<PersonalAccessToken>> {
        let store = self.store.lock().unwrap();

        let cursor_dt = page
            .cursor
            .as_deref()
            .and_then(|c| Cursor::decode_utc_timestamp(c).ok());

        let mut pats: Vec<PersonalAccessToken> = store
            .values()
            .filter(|p| {
                if let Some(ref tid) = filter.tenant_id {
                    if p.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(ref status) = filter.status {
                    if status == "active" && !p.is_active() {
                        return false;
                    }
                    if status == "revoked" && !p.revoked {
                        return false;
                    }
                }
                if let Some(role) = filter.role {
                    if p.role != role {
                        return false;
                    }
                }
                if let Some(ref search) = filter.search {
                    let term = search.to_lowercase();
                    if !p.name.to_lowercase().contains(&term)
                        && !p.token_prefix.to_lowercase().contains(&term)
                    {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if p.created_at <= after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if p.created_at >= before {
                        return false;
                    }
                }
                if let Some(cursor) = cursor_dt {
                    match sort {
                        Sort::CreatedAtAsc => {
                            if p.created_at <= cursor {
                                return false;
                            }
                        }
                        _ => {
                            if p.created_at >= cursor {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        match sort {
            Sort::CreatedAtAsc => pats.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            _ => pats.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        pats.truncate(page.limit as usize);
        Ok(pats)
    }

    async fn count(&self, filter: &PatFilter) -> Outcome<u64> {
        let store = self.store.lock().unwrap();
        let count = store
            .values()
            .filter(|p| {
                if let Some(ref tid) = filter.tenant_id {
                    if p.tenant_id != *tid {
                        return false;
                    }
                }
                if let Some(ref status) = filter.status {
                    if status == "active" && !p.is_active() {
                        return false;
                    }
                    if status == "revoked" && !p.revoked {
                        return false;
                    }
                }
                if let Some(role) = filter.role {
                    if p.role != role {
                        return false;
                    }
                }
                if let Some(ref search) = filter.search {
                    let term = search.to_lowercase();
                    if !p.name.to_lowercase().contains(&term)
                        && !p.token_prefix.to_lowercase().contains(&term)
                    {
                        return false;
                    }
                }
                if let Some(after) = filter.created_after {
                    if p.created_at <= after {
                        return false;
                    }
                }
                if let Some(before) = filter.created_before {
                    if p.created_at >= before {
                        return false;
                    }
                }
                true
            })
            .count();
        Ok(count as u64)
    }

    async fn create(&self, pat: &PersonalAccessToken) -> Outcome<PersonalAccessToken> {
        let mut store = self.store.lock().unwrap();
        store.insert(pat.id, pat.clone());
        Ok(pat.clone())
    }

    async fn get_by_id(&self, tenant_id: &str, id: Uuid) -> Outcome<Option<PersonalAccessToken>> {
        let store = self.store.lock().unwrap();
        let item = store.get(&id).cloned();
        Ok(item.filter(|p| p.tenant_id == tenant_id))
    }

    async fn get_batch(&self, tenant_id: &str, ids: &[Uuid]) -> Outcome<Vec<PersonalAccessToken>> {
        let store = self.store.lock().unwrap();
        let mut result = Vec::new();
        for id in ids {
            if let Some(pat) = store.get(id) {
                if pat.tenant_id == tenant_id {
                    result.push(pat.clone());
                }
            }
        }
        Ok(result)
    }

    async fn get_by_hash(&self, token_hash: &str) -> Outcome<Option<PersonalAccessToken>> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .find(|p| p.token_hash == token_hash)
            .cloned())
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> Outcome<Vec<PersonalAccessToken>> {
        let store = self.store.lock().unwrap();
        Ok(store
            .values()
            .filter(|p| p.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn revoke(&self, tenant_id: &str, id: Uuid) -> Outcome<()> {
        let mut store = self.store.lock().unwrap();
        if let Some(p) = store.get_mut(&id) {
            if p.tenant_id != tenant_id {
                return Err(PatRepositoryError::NotFound.into_errors());
            }
            p.revoked = true;
            Ok(())
        } else {
            Err(PatRepositoryError::NotFound.into_errors())
        }
    }

    async fn update_last_used(&self, id: Uuid) -> Outcome<()> {
        if let Some(p) = self.store.lock().unwrap().get_mut(&id) {
            p.last_used_at = Some(Utc::now());
        }
        Ok(())
    }
}
