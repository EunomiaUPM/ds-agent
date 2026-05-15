use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use base64::Engine;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::refresh_token::RefreshTokenRepository;
use crate::data::repositories::user::{UserRepository, UserRepositoryError};
use crate::entities::query::{Page, Sort, UserFilter};
use crate::entities::refresh_token::RefreshToken;
use crate::entities::role::Role;
use crate::entities::user::User;

// ── User ──────────────────────────────────────────────────────────────────────

pub(crate) struct InMemoryUserRepository {
    store: Arc<Mutex<HashMap<String, User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait::async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn get_all(&self, filter: &UserFilter, page: &Page, sort: &Sort) -> Outcome<Vec<User>> {
        let store = self.store.lock().unwrap();

        let cursor_dt = page.cursor.as_ref().and_then(|c| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(c).ok()
                .and_then(|b| String::from_utf8(b).ok())
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc))
        });

        let mut users: Vec<User> = store.values().filter(|u| {
            if let Some(role) = filter.role { if u.role != role { return false; } }
            if let Some(ref email) = filter.email { if !u.email.contains(email.as_str()) { return false; } }
            if let Some(after) = filter.created_after { if u.created_at <= after { return false; } }
            if let Some(before) = filter.created_before { if u.created_at >= before { return false; } }
            if let Some(cursor) = cursor_dt {
                match sort {
                    Sort::CreatedAtAsc => { if u.created_at <= cursor { return false; } }
                    Sort::CreatedAtDesc => { if u.created_at >= cursor { return false; } }
                }
            }
            true
        }).cloned().collect();

        match sort {
            Sort::CreatedAtAsc => users.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            Sort::CreatedAtDesc => users.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        users.truncate(page.limit as usize);
        Ok(users)
    }

    async fn get_by_tenant_id(&self, tenant_id: &str) -> Outcome<Option<User>> {
        Ok(self.store.lock().unwrap().get(tenant_id).cloned())
    }

    async fn get_by_email(&self, email: &str) -> Outcome<Option<User>> {
        Ok(self.store.lock().unwrap().values().find(|u| u.email == email).cloned())
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
        role: Option<Role>,
        extra_fields: Option<serde_json::Value>,
    ) -> Outcome<User> {
        let mut store = self.store.lock().unwrap();
        let user = store
            .get_mut(tenant_id)
            .ok_or_else(|| UserRepositoryError::NotFound.into_errors())?;
        if let Some(e) = email { user.email = e; }
        if let Some(r) = role { user.role = r; }
        if let Some(f) = extra_fields { user.extra_fields = f; }
        Ok(user.clone())
    }

    async fn delete(&self, tenant_id: &str) -> Outcome<()> {
        self.store.lock().unwrap().remove(tenant_id);
        Ok(())
    }
}

// ── RefreshToken ──────────────────────────────────────────────────────────────

pub(crate) struct InMemoryRefreshTokenRepository {
    store: Arc<Mutex<HashMap<Uuid, RefreshToken>>>,
}

impl InMemoryRefreshTokenRepository {
    pub fn new() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait::async_trait]
impl RefreshTokenRepository for InMemoryRefreshTokenRepository {
    async fn create(&self, token: &RefreshToken) -> Outcome<RefreshToken> {
        self.store.lock().unwrap().insert(token.id, token.clone());
        Ok(token.clone())
    }

    async fn get_by_jti(&self, jti: &str) -> Outcome<Option<RefreshToken>> {
        Ok(self.store.lock().unwrap().values().find(|t| t.jti == jti).cloned())
    }

    async fn revoke(&self, id: Uuid) -> Outcome<()> {
        if let Some(t) = self.store.lock().unwrap().get_mut(&id) {
            t.revoked = true;
        }
        Ok(())
    }

    async fn revoke_all_for_tenant(&self, tenant_id: &str) -> Outcome<()> {
        self.store.lock().unwrap().values_mut().filter(|t| t.tenant_id == tenant_id).for_each(|t| {
            t.revoked = true;
        });
        Ok(())
    }
}
