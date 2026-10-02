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

//! TokenService with mocked repositories, split by grant. Helpers build users and clients with
//! real Argon2 hashes and keep refresh records in memory so tokens can be refreshed and revoked.

mod authorization_code;
mod client_credentials;
mod introspection;
mod password_grant;
mod refresh;

use std::sync::{Arc, Mutex};

use chrono::Utc;
use oauth::config::OAuthConfig;
use oauth::data::repositories::auth_code::MockAuthCodeRepository;
use oauth::data::repositories::client::MockClientRepository;
use oauth::data::repositories::pat::MockPatRepository;
use oauth::data::repositories::token::MockTokenRepository;
use oauth::data::repositories::user::MockUserRepository;
use oauth::entities::client::Client;
use oauth::entities::refresh_token::RefreshToken;
use oauth::entities::role::RbacRole;
use oauth::entities::user::User;
use oauth::services::password;
use oauth::services::token_service::service::TokenService;
use serde_json::json;

const SECRET: &str = "test-signing-secret";
const ISSUER: &str = "https://auth.test";
const AUDIENCE: &str = "ds-protocol";

fn config() -> OAuthConfig {
    OAuthConfig::new(SECRET, ISSUER, AUDIENCE)
}

/// User of `tenant` whose password is `password`.
fn user(tenant: &str, email: &str, password: &str, role: RbacRole) -> User {
    let (hash, salt) = password::hash_password(password).unwrap();
    User {
        tenant_id: tenant.to_string(),
        email: email.to_string(),
        password_hash: hash,
        password_salt: salt,
        role,
        created_at: Utc::now(),
        extra_fields: json!({ "name": "Alice" }),
    }
}

/// Client of `tenant` whose secret is `secret`, allowed `scopes`.
fn client(client_id: &str, tenant: &str, secret: &str, scopes: &[&str]) -> Client {
    let (hash, _) = password::hash_password(secret).unwrap();
    Client {
        client_id: client_id.to_string(),
        tenant_id: tenant.to_string(),
        client_secret_hash: hash,
        client_name: client_id.to_string(),
        role: RbacRole::Owner,
        scopes: scopes.iter().map(ToString::to_string).collect(),
        created_at: Utc::now(),
    }
}

/// Mocked repositories, configured by each test before building the service.
#[derive(Default)]
struct Repos {
    users: MockUserRepository,
    refresh: MockTokenRepository,
    clients: MockClientRepository,
    codes: MockAuthCodeRepository,
    pats: MockPatRepository,
}

impl Repos {
    fn service(self) -> TokenService {
        self.service_with(config())
    }

    fn service_with(self, config: OAuthConfig) -> TokenService {
        TokenService::new(
            Arc::new(self.users),
            Arc::new(self.refresh),
            Arc::new(self.clients),
            Arc::new(self.codes),
            Arc::new(self.pats),
            config,
        )
    }

    /// The user is found by email and by tenant id.
    fn with_user(mut self, user: User) -> Self {
        let by_email = user.clone();
        self.users
            .expect_get_by_email()
            .returning(move |e| Ok((e == by_email.email).then(|| by_email.clone())));
        let by_tenant = user;
        self.users
            .expect_get_by_tenant_id()
            .returning(move |t| Ok((t == by_tenant.tenant_id).then(|| by_tenant.clone())));
        self
    }

    /// Refresh records live in the returned list: created, looked up by jti and revoked there.
    fn with_refresh_store(mut self) -> (Self, Arc<Mutex<Vec<RefreshToken>>>) {
        let store: Arc<Mutex<Vec<RefreshToken>>> = Arc::default();
        let created = store.clone();
        self.refresh.expect_create().returning(move |t| {
            created.lock().unwrap().push(t.clone());
            Ok(t.clone())
        });
        let found = store.clone();
        self.refresh
            .expect_get_by_jti()
            .returning(move |jti| Ok(found.lock().unwrap().iter().find(|t| t.jti == jti).cloned()));
        let revoked = store.clone();
        self.refresh.expect_revoke().returning(move |id| {
            for t in revoked.lock().unwrap().iter_mut().filter(|t| t.id == id) {
                t.revoked = true;
            }
            Ok(())
        });
        (self, store)
    }
}
