use std::sync::Arc;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::config::OAuthConfig;
use crate::data::repo::refresh_token_repo::RefreshTokenRepoTrait;
use crate::data::repo::user_repo::UserRepoTrait;
use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::jwt_claims::{Claims, IdTokenClaims, RefreshClaims};
use crate::entities::oidc::{TokenResponse, UserInfo};
use crate::entities::query::UserView;
use crate::entities::refresh_token::RefreshToken;
use crate::entities::role::Role;
use crate::entities::user::User;
use crate::services::AuthServiceTrait;

pub struct AuthService {
    user_repo: Arc<dyn UserRepoTrait>,
    refresh_token_repo: Arc<dyn RefreshTokenRepoTrait>,
    config: OAuthConfig,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepoTrait>,
        refresh_token_repo: Arc<dyn RefreshTokenRepoTrait>,
        config: OAuthConfig,
    ) -> Self {
        Self { user_repo, refresh_token_repo, config }
    }

    // Access token ────────────────────────────────────────────────────────
    fn encode_access(&self, tenant_id: &str, role: Role) -> Outcome<String> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: tenant_id.to_string(),
            role,
            iat: now,
            exp: now + self.config.access_token_ttl_secs,
        };
        self.sign(&claims)
    }

    pub fn decode_access(&self, token: &str) -> Outcome<Claims> {
        self.verify(token)
    }

    // ID token (OIDC) ─────────────────────────────────────────────────────
    fn encode_id_token(
        &self,
        tenant_id: &str,
        email: &str,
        role: Role,
        extra: serde_json::Map<String, serde_json::Value>,
    ) -> Outcome<String> {
        let now = Utc::now().timestamp();
        let claims = IdTokenClaims {
            iss: self.config.issuer.clone(),
            sub: tenant_id.to_string(),
            aud: self.config.audience.clone(),
            exp: now + self.config.access_token_ttl_secs,
            iat: now,
            email: email.to_string(),
            role,
            extra,
        };
        self.sign(&claims)
    }

    // Refresh token ───────────────────────────────────────────────────────
    async fn mint_refresh(&self, tenant_id: &str, role: Role) -> Outcome<String> {
        let jti = Uuid::new_v4().to_string();
        let now = Utc::now();
        let record = RefreshToken {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            jti: jti.clone(),
            expires_at: now + chrono::Duration::seconds(self.config.refresh_token_ttl_secs),
            created_at: now,
            revoked: false,
        };
        self.refresh_token_repo.create(&record).await?;

        let claims = RefreshClaims {
            sub: tenant_id.to_string(),
            role,
            jti,
            iat: now.timestamp(),
            exp: now.timestamp() + self.config.refresh_token_ttl_secs,
        };
        self.sign(&claims)
    }

    fn decode_refresh(&self, token: &str) -> Outcome<RefreshClaims> {
        self.verify(token)
    }

    // JWT helpers ─────────────────────────────────────────────────────────
    fn sign<T: serde::Serialize>(&self, claims: &T) -> Outcome<String> {
        jsonwebtoken::encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Errors::crazy("JWT encoding failed", Some(Box::new(e))))
    }

    fn verify<T: serde::de::DeserializeOwned>(&self, token: &str) -> Outcome<T> {
        let mut v = Validation::new(Algorithm::HS256);
        v.validate_exp = true;
        v.validate_aud = false;
        jsonwebtoken::decode::<T>(token, &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()), &v)
            .map(|d| d.claims)
            .map_err(|e| {
                Errors::format(BadFormat::Received, "invalid or expired token", Some(Box::new(e)))
            })
    }
}

#[async_trait::async_trait]
impl AuthServiceTrait for AuthService {
    async fn issue_token(&self, email: &str, password: &str) -> Outcome<TokenResponse> {
        let user = self
            .user_repo
            .get_by_email(email)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "invalid credentials", None))?;

        let parsed = PasswordHash::new(&user.password_hash)
            .map_err(|e| Errors::crazy("password hash parse error", Some(e.to_string().into())))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| Errors::format(BadFormat::Received, "invalid credentials", None))?;

        let extra = as_map(user.extra_fields.clone());
        let access_token = self.encode_access(&user.tenant_id, user.role)?;
        let id_token = self.encode_id_token(&user.tenant_id, &user.email, user.role, extra)?;
        let refresh_token = self.mint_refresh(&user.tenant_id, user.role).await?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            refresh_token,
            id_token,
        })
    }

    async fn refresh_token(&self, refresh_jwt: &str) -> Outcome<TokenResponse> {
        let rc = self.decode_refresh(refresh_jwt)?;

        let record = self
            .refresh_token_repo
            .get_by_jti(&rc.jti)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "unknown refresh token", None))?;

        if record.revoked {
            return Err(Errors::format(BadFormat::Received, "refresh token revoked", None));
        }
        self.refresh_token_repo.revoke(record.id).await?;

        let user = self
            .user_repo
            .get_by_tenant_id(&rc.sub)
            .await?
            .ok_or_else(|| Errors::crazy("tenant not found for refresh token", None))?;

        let extra = as_map(user.extra_fields.clone());
        let access_token = self.encode_access(&user.tenant_id, user.role)?;
        let id_token = self.encode_id_token(&user.tenant_id, &user.email, user.role, extra)?;
        let new_refresh = self.mint_refresh(&user.tenant_id, user.role).await?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
            refresh_token: new_refresh,
            id_token,
        })
    }

    async fn validate_token(&self, access_token: &str) -> Outcome<Claims> {
        self.decode_access(access_token)
    }

    async fn userinfo(&self, access_token: &str) -> Outcome<UserInfo> {
        let claims = self.decode_access(access_token)?;
        let user = self
            .user_repo
            .get_by_tenant_id(&claims.sub)
            .await?
            .ok_or_else(|| Errors::crazy("user not found", None))?;
        Ok(UserInfo {
            sub: user.tenant_id,
            email: user.email,
            role: user.role,
            extra: as_map(user.extra_fields),
        })
    }

    async fn revoke_refresh_token(&self, refresh_jwt: &str) -> Outcome<()> {
        let rc = self.decode_refresh(refresh_jwt)?;
        let record = self
            .refresh_token_repo
            .get_by_jti(&rc.jti)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "unknown refresh token", None))?;
        self.refresh_token_repo.revoke(record.id).await
    }

    // User CRUD ───────────────────────────────────────────────────────────

    async fn list_users(&self) -> Outcome<Vec<UserView>> {
        let users = self.user_repo.get_all().await?;
        Ok(users.into_iter().map(UserView::from).collect())
    }

    async fn get_user(&self, tenant_id: &str) -> Outcome<UserView> {
        self.user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .map(UserView::from)
            .ok_or_else(|| Errors::format(BadFormat::Received, "user not found", None))
    }

    async fn create_user(&self, cmd: &CreateUserCommand) -> Outcome<UserView> {
        if self.user_repo.get_by_tenant_id(&cmd.tenant_id).await?.is_some() {
            return Err(Errors::format(BadFormat::Received, "tenant_id already in use", None));
        }
        if self.user_repo.get_by_email(&cmd.email).await?.is_some() {
            return Err(Errors::format(BadFormat::Received, "email already in use", None));
        }
        let user = User {
            tenant_id: cmd.tenant_id.clone(),
            email: cmd.email.clone(),
            password_hash: hash_password(&cmd.password)?,
            role: cmd.role,
            created_at: Utc::now(),
            extra_fields: cmd.extra_fields.clone(),
        };
        Ok(UserView::from(self.user_repo.create(&user).await?))
    }

    async fn patch_user(&self, tenant_id: &str, cmd: &PatchUserCommand) -> Outcome<UserView> {
        Ok(UserView::from(self.user_repo.patch(tenant_id, cmd).await?))
    }

    async fn delete_user(&self, tenant_id: &str) -> Outcome<()> {
        self.user_repo.delete(tenant_id).await
    }
}

// Helpers ────────────────────────────────────────────────────────────────────

fn as_map(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match v {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    }
}

pub fn hash_password(password: &str) -> Outcome<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| Errors::crazy("password hashing failed", Some(e.to_string().into())))
}
