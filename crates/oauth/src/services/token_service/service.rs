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

use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::config::OAuthConfig;
use crate::data::repositories::auth_code::AuthCodeRepository;
use crate::data::repositories::client::ClientRepository;
use crate::data::repositories::pat::PatRepository;
use crate::data::repositories::token::TokenRepository;
use crate::data::repositories::user::UserRepository;
use crate::entities::auth_code::AuthCode;
use crate::entities::pat::PersonalAccessToken;
use crate::entities::refresh_token::RefreshToken;
use crate::entities::role::RbacRole;
use crate::http::forms::IntrospectResponse;
use crate::services::password;
use crate::services::token_service::jwt::{
    AccessClaims, IdTokenClaims, JwtAssertionClaims, RefreshClaims, as_map,
};
use crate::services::token_service::views::TokenResponse;
use crate::services::token_service::{Claims, OauthTokenValidator, TokenServiceTrait};

pub(crate) struct TokenService {
    user_repo: Arc<dyn UserRepository>,
    refresh_repo: Arc<dyn TokenRepository>,
    client_repo: Arc<dyn ClientRepository>,
    auth_code_repo: Arc<dyn AuthCodeRepository>,
    pat_repo: Arc<dyn PatRepository>,
    config: OAuthConfig,
}

impl TokenService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_repo: Arc<dyn TokenRepository>,
        client_repo: Arc<dyn ClientRepository>,
        auth_code_repo: Arc<dyn AuthCodeRepository>,
        pat_repo: Arc<dyn PatRepository>,
        config: OAuthConfig,
    ) -> Self {
        Self {
            user_repo,
            refresh_repo,
            client_repo,
            auth_code_repo,
            pat_repo,
            config,
        }
    }

    fn sign<T: Serialize>(&self, claims: &T) -> Outcome<String> {
        jsonwebtoken::encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Errors::crazy("JWT encoding failed", Some(Box::new(e))))
    }

    fn verify<T: for<'de> Deserialize<'de>>(&self, token: &str) -> Outcome<T> {
        let mut v = Validation::new(Algorithm::HS256);
        v.validate_exp = true;
        v.validate_aud = false;
        jsonwebtoken::decode::<T>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &v,
        )
        .map(|d| d.claims)
        .map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "invalid or expired token",
                Some(Box::new(e)),
            )
        })
    }

    fn encode_access(
        &self,
        tenant_id: &str,
        role: RbacRole,
        scope: Option<String>,
        client_id: Option<String>,
    ) -> Outcome<String> {
        let now = Utc::now().timestamp();
        self.sign(&AccessClaims {
            sub: tenant_id.to_string(),
            role,
            iat: now,
            exp: now + self.config.access_token_ttl_secs,
            scope,
            client_id,
        })
    }

    fn encode_id_token(
        &self,
        tenant_id: &str,
        email: &str,
        role: RbacRole,
        extra: serde_json::Map<String, serde_json::Value>,
    ) -> Outcome<String> {
        let now = Utc::now().timestamp();
        self.sign(&IdTokenClaims {
            iss: self.config.issuer.clone(),
            sub: tenant_id.to_string(),
            aud: self.config.audience.clone(),
            exp: now + self.config.access_token_ttl_secs,
            iat: now,
            email: email.to_string(),
            role,
            extra,
        })
    }

    async fn mint_refresh(&self, tenant_id: &str, role: RbacRole) -> Outcome<String> {
        let jti = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.refresh_repo
            .create(&RefreshToken {
                id: Uuid::new_v4(),
                tenant_id: tenant_id.to_string(),
                jti: jti.clone(),
                expires_at: now + chrono::Duration::seconds(self.config.refresh_token_ttl_secs),
                created_at: now,
                revoked: false,
            })
            .await?;
        self.sign(&RefreshClaims {
            sub: tenant_id.to_string(),
            role,
            jti,
            iat: now.timestamp(),
            exp: now.timestamp() + self.config.refresh_token_ttl_secs,
        })
    }
}

#[async_trait::async_trait]
impl OauthTokenValidator for TokenService {
    async fn validate_token(&self, access_token: &str) -> Outcome<Claims> {
        if access_token.starts_with("pat_") {
            let hash = PersonalAccessToken::hash_token(access_token);
            let pat = self
                .pat_repo
                .get_by_hash(&hash)
                .await?
                .ok_or_else(|| Errors::unauthorized("invalid or revoked PAT", None))?;
            if !pat.is_active() {
                return Err(Errors::unauthorized("PAT is expired or revoked", None));
            }
            let _ = self.pat_repo.update_last_used(pat.id).await;
            let now = Utc::now().timestamp();
            let exp = pat
                .expires_at
                .map(|dt| dt.timestamp())
                .unwrap_or(now + 31_536_000);
            return Ok(Claims {
                sub: pat.tenant_id,
                role: pat.role,
                iat: pat.created_at.timestamp(),
                exp,
            });
        }

        let ac: AccessClaims = self
            .verify(access_token)
            .map_err(|e| Errors::unauthorized("invalid or expired token", Some(Box::new(e))))?;
        Ok(Claims {
            sub: ac.sub,
            role: ac.role,
            iat: ac.iat,
            exp: ac.exp,
        })
    }
}

#[async_trait::async_trait]
impl TokenServiceTrait for TokenService {
    async fn issue_token(&self, email: &str, password: &str) -> Outcome<TokenResponse> {
        self.issue_token_with_scope(email, password, None).await
    }

    async fn issue_token_with_scope(
        &self,
        email: &str,
        password: &str,
        scope: Option<&str>,
    ) -> Outcome<TokenResponse> {
        let user = self
            .user_repo
            .get_by_email(email)
            .await?
            .or(self.user_repo.get_by_tenant_id(email).await?)
            .ok_or_else(|| Errors::format(BadFormat::Received, "invalid credentials", None))?;

        password::verify_password(password, &user.password_hash)?;

        let extra = as_map(user.extra_fields);
        let scope_str = scope.map(ToString::to_string);
        Ok(TokenResponse {
            access_token: self.encode_access(
                &user.tenant_id,
                user.role,
                scope_str.clone(),
                None,
            )?,
            id_token: Some(self.encode_id_token(&user.tenant_id, &user.email, user.role, extra)?),
            refresh_token: Some(self.mint_refresh(&user.tenant_id, user.role).await?),
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            scope: scope_str,
        })
    }

    async fn issue_client_credentials_token(
        &self,
        client_id: &str,
        client_secret: &str,
        requested_scope: Option<&str>,
    ) -> Outcome<TokenResponse> {
        let client = self
            .client_repo
            .get_by_client_id(client_id)
            .await?
            .ok_or_else(|| {
                Errors::format(BadFormat::Received, "invalid client credentials", None)
            })?;

        password::verify_password(client_secret, &client.client_secret_hash)?;

        let scope_str = match requested_scope {
            Some(s) if !s.trim().is_empty() => {
                let requested: Vec<&str> = s.split_whitespace().collect();
                if !client.scopes.is_empty() {
                    for req in &requested {
                        if !client.scopes.iter().any(|allowed| allowed == req) {
                            return Err(Errors::format(
                                BadFormat::Received,
                                format!("requested scope '{req}' exceeds client authorized scopes"),
                                None,
                            ));
                        }
                    }
                }
                Some(s.to_string())
            }
            _ => {
                if client.scopes.is_empty() {
                    None
                } else {
                    Some(client.scopes.join(" "))
                }
            }
        };

        let access_token = self.encode_access(
            &client.client_id,
            client.role,
            scope_str.clone(),
            Some(client.client_id.clone()),
        )?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            refresh_token: None,
            id_token: None,
            scope: scope_str,
        })
    }

    async fn issue_authorization_code(
        &self,
        client_id: &str,
        redirect_uri: Option<&str>,
        scope: Option<&str>,
        code_challenge: &str,
        code_challenge_method: Option<&str>,
        user_id: Option<&str>,
    ) -> Outcome<String> {
        let _client = self
            .client_repo
            .get_by_client_id(client_id)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "client not found", None))?;

        if code_challenge.trim().is_empty() {
            return Err(Errors::format(
                BadFormat::Received,
                "code_challenge is required for PKCE",
                None,
            ));
        }

        let method = code_challenge_method.unwrap_or("S256");
        if method != "S256" && method != "plain" {
            return Err(Errors::format(
                BadFormat::Received,
                "unsupported code_challenge_method: only S256 and plain are supported",
                None,
            ));
        }

        let uid = user_id.ok_or_else(|| {
            Errors::format(BadFormat::Received, "user identification required", None)
        })?;
        let user = self
            .user_repo
            .get_by_tenant_id(uid)
            .await?
            .or(self.user_repo.get_by_email(uid).await?)
            .ok_or_else(|| Errors::format(BadFormat::Received, "user not found", None))?;

        let scopes: Vec<String> = scope
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let code = {
            let mut rng = rand::rng();
            let random_bytes: [u8; 32] = rng.random();
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
        };

        let auth_code = AuthCode {
            code: code.clone(),
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.map(ToString::to_string),
            tenant_id: user.tenant_id,
            role: user.role,
            scopes,
            code_challenge: code_challenge.to_string(),
            code_challenge_method: method.to_string(),
            expires_at: Utc::now() + chrono::Duration::seconds(600),
            used: false,
        };

        self.auth_code_repo.save(&auth_code).await?;
        Ok(code)
    }

    async fn exchange_authorization_code(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: Option<&str>,
        client_id: Option<&str>,
    ) -> Outcome<TokenResponse> {
        let record = self
            .auth_code_repo
            .get_by_code(code)
            .await?
            .ok_or_else(|| {
                Errors::format(BadFormat::Received, "invalid authorization code", None)
            })?;

        if record.used {
            return Err(Errors::format(
                BadFormat::Received,
                "authorization code has already been used",
                None,
            ));
        }
        if record.expires_at <= Utc::now() {
            return Err(Errors::format(
                BadFormat::Received,
                "authorization code expired",
                None,
            ));
        }
        if let Some(cid) = client_id {
            if record.client_id != cid {
                return Err(Errors::format(BadFormat::Received, "client mismatch", None));
            }
        }
        if let Some(r_uri) = record.redirect_uri.as_deref() {
            if redirect_uri != Some(r_uri) {
                return Err(Errors::format(
                    BadFormat::Received,
                    "redirect_uri mismatch",
                    None,
                ));
            }
        }
        if !record.verify_pkce(code_verifier) {
            return Err(Errors::format(
                BadFormat::Received,
                "invalid PKCE code_verifier",
                None,
            ));
        }

        self.auth_code_repo.mark_used(code).await?;

        let scope_str = if record.scopes.is_empty() {
            None
        } else {
            Some(record.scopes.join(" "))
        };

        let access_token = self.encode_access(
            &record.tenant_id,
            record.role,
            scope_str.clone(),
            Some(record.client_id.clone()),
        )?;

        let refresh_token = Some(self.mint_refresh(&record.tenant_id, record.role).await?);

        let id_token = if let Ok(Some(u)) = self.user_repo.get_by_tenant_id(&record.tenant_id).await
        {
            Some(self.encode_id_token(&u.tenant_id, &u.email, u.role, as_map(u.extra_fields))?)
        } else {
            None
        };

        Ok(TokenResponse {
            access_token,
            id_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            scope: scope_str,
        })
    }

    async fn issue_jwt_bearer_token(
        &self,
        assertion: &str,
        scope: Option<&str>,
    ) -> Outcome<TokenResponse> {
        let claims: JwtAssertionClaims = self.verify(assertion)?;

        let (tenant_id, role, client_scopes, client_id) =
            if let Ok(Some(client)) = self.client_repo.get_by_client_id(&claims.iss).await {
                (
                    client.client_id.clone(),
                    client.role,
                    client.scopes,
                    Some(client.client_id),
                )
            } else if let Ok(Some(client)) = self.client_repo.get_by_client_id(&claims.sub).await {
                (
                    client.client_id.clone(),
                    client.role,
                    client.scopes,
                    Some(client.client_id),
                )
            } else if let Ok(Some(user)) = self.user_repo.get_by_tenant_id(&claims.sub).await {
                (user.tenant_id, user.role, vec![], None)
            } else {
                (
                    claims.sub.clone(),
                    RbacRole::Reader,
                    vec![],
                    Some(claims.iss.clone()),
                )
            };

        let requested = scope.or(claims.scope.as_deref());
        let scope_str = match requested {
            Some(s) if !s.trim().is_empty() => {
                if !client_scopes.is_empty() {
                    for req in s.split_whitespace() {
                        if !client_scopes.iter().any(|allowed| allowed == req) {
                            return Err(Errors::format(
                                BadFormat::Received,
                                format!("requested scope '{req}' exceeds authorized scopes"),
                                None,
                            ));
                        }
                    }
                }
                Some(s.to_string())
            }
            _ => {
                if client_scopes.is_empty() {
                    None
                } else {
                    Some(client_scopes.join(" "))
                }
            }
        };

        let access_token = self.encode_access(&tenant_id, role, scope_str.clone(), client_id)?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            refresh_token: None,
            id_token: None,
            scope: scope_str,
        })
    }

    async fn refresh_token(&self, refresh_jwt: &str) -> Outcome<TokenResponse> {
        self.refresh_token_with_scope(refresh_jwt, None).await
    }

    async fn refresh_token_with_scope(
        &self,
        refresh_jwt: &str,
        scope: Option<&str>,
    ) -> Outcome<TokenResponse> {
        let rc: RefreshClaims = self.verify(refresh_jwt)?;
        let record = self
            .refresh_repo
            .get_by_jti(&rc.jti)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "unknown refresh token", None))?;
        if record.revoked {
            return Err(Errors::format(
                BadFormat::Received,
                "refresh token revoked",
                None,
            ));
        }
        self.refresh_repo.revoke(record.id).await?;

        let user = self
            .user_repo
            .get_by_tenant_id(&rc.sub)
            .await?
            .ok_or_else(|| Errors::crazy("user not found for refresh token", None))?;

        let extra = as_map(user.extra_fields);
        let scope_str = scope.map(ToString::to_string);
        Ok(TokenResponse {
            access_token: self.encode_access(
                &user.tenant_id,
                user.role,
                scope_str.clone(),
                None,
            )?,
            id_token: Some(self.encode_id_token(&user.tenant_id, &user.email, user.role, extra)?),
            refresh_token: Some(self.mint_refresh(&user.tenant_id, user.role).await?),
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            scope: scope_str,
        })
    }

    async fn revoke_refresh_token(&self, refresh_jwt: &str) -> Outcome<()> {
        let rc: RefreshClaims = self.verify(refresh_jwt)?;
        let record = self
            .refresh_repo
            .get_by_jti(&rc.jti)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "unknown refresh token", None))?;
        self.refresh_repo.revoke(record.id).await
    }

    async fn revoke_token(&self, token: &str, hint: Option<&str>) -> Outcome<()> {
        if token.starts_with("pat_") {
            let hash = PersonalAccessToken::hash_token(token);
            if let Ok(Some(pat)) = self.pat_repo.get_by_hash(&hash).await {
                let _ = self.pat_repo.revoke(pat.id).await;
            }
            return Ok(());
        }
        if hint != Some("access_token") {
            if let Ok(rc) = self.verify::<RefreshClaims>(token) {
                if let Ok(Some(rec)) = self.refresh_repo.get_by_jti(&rc.jti).await {
                    let _ = self.refresh_repo.revoke(rec.id).await;
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    async fn introspect_token(
        &self,
        token: &str,
        hint: Option<&str>,
    ) -> Outcome<IntrospectResponse> {
        let now = Utc::now().timestamp();

        if token.starts_with("pat_") {
            let hash = PersonalAccessToken::hash_token(token);
            if let Ok(Some(pat)) = self.pat_repo.get_by_hash(&hash).await {
                if pat.is_active() {
                    return Ok(IntrospectResponse {
                        active: true,
                        scope: if pat.scopes.is_empty() {
                            None
                        } else {
                            Some(pat.scopes.join(" "))
                        },
                        client_id: None,
                        sub: Some(pat.tenant_id),
                        exp: pat.expires_at.map(|dt| dt.timestamp()),
                        iat: Some(pat.created_at.timestamp()),
                        token_type: Some("pat".to_string()),
                        role: Some(pat.role.to_string()),
                    });
                }
            }
            return Ok(IntrospectResponse::inactive());
        }

        if hint != Some("refresh_token") {
            if let Ok(ac) = self.verify::<AccessClaims>(token) {
                if ac.exp > now {
                    return Ok(IntrospectResponse {
                        active: true,
                        scope: ac.scope,
                        client_id: ac.client_id,
                        sub: Some(ac.sub),
                        exp: Some(ac.exp),
                        iat: Some(ac.iat),
                        token_type: Some("Bearer".to_string()),
                        role: Some(ac.role.to_string()),
                    });
                }
            }
        }

        if hint != Some("access_token") {
            if let Ok(rc) = self.verify::<RefreshClaims>(token) {
                if rc.exp > now {
                    if let Ok(Some(rec)) = self.refresh_repo.get_by_jti(&rc.jti).await {
                        if !rec.revoked {
                            return Ok(IntrospectResponse {
                                active: true,
                                scope: None,
                                client_id: None,
                                sub: Some(rc.sub),
                                exp: Some(rc.exp),
                                iat: Some(rc.iat),
                                token_type: Some("refresh_token".to_string()),
                                role: Some(rc.role.to_string()),
                            });
                        }
                    }
                }
            }
        }

        Ok(IntrospectResponse::inactive())
    }
}
