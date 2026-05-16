use std::sync::Arc;

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::config::OAuthConfig;
use crate::data::repositories::refresh_token::RefreshTokenRepository;
use crate::data::repositories::user::UserRepository;
use crate::entities::refresh_token::RefreshToken;
use crate::entities::role::Role;
use crate::services::password;
use crate::services::token_service::jwt::{AccessClaims, IdTokenClaims, RefreshClaims, as_map};
use crate::services::token_service::views::TokenResponse;
use crate::services::token_service::{Claims, TokenServiceTrait, TokenValidator};

pub(crate) struct TokenService {
    user_repo: Arc<dyn UserRepository>,
    refresh_repo: Arc<dyn RefreshTokenRepository>,
    config: OAuthConfig,
}

impl TokenService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_repo: Arc<dyn RefreshTokenRepository>,
        config: OAuthConfig,
    ) -> Self {
        Self {
            user_repo,
            refresh_repo,
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

    fn encode_access(&self, tenant_id: &str, role: Role) -> Outcome<String> {
        let now = Utc::now().timestamp();
        self.sign(&AccessClaims {
            sub: tenant_id.to_string(),
            role,
            iat: now,
            exp: now + self.config.access_token_ttl_secs,
        })
    }

    fn encode_id_token(
        &self,
        tenant_id: &str,
        email: &str,
        role: Role,
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

    async fn mint_refresh(&self, tenant_id: &str, role: Role) -> Outcome<String> {
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
impl TokenValidator for TokenService {
    async fn validate_token(&self, access_token: &str) -> Outcome<Claims> {
        let ac: AccessClaims = self.verify(access_token)?;
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
        let user = self
            .user_repo
            .get_by_email(email)
            .await?
            .ok_or_else(|| Errors::format(BadFormat::Received, "invalid credentials", None))?;

        password::verify_password(password, &user.password_hash)?;

        let extra = as_map(user.extra_fields);
        Ok(TokenResponse {
            access_token: self.encode_access(&user.tenant_id, user.role)?,
            id_token: self.encode_id_token(&user.tenant_id, &user.email, user.role, extra)?,
            refresh_token: self.mint_refresh(&user.tenant_id, user.role).await?,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
        })
    }

    async fn refresh_token(&self, refresh_jwt: &str) -> Outcome<TokenResponse> {
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
        Ok(TokenResponse {
            access_token: self.encode_access(&user.tenant_id, user.role)?,
            id_token: self.encode_id_token(&user.tenant_id, &user.email, user.role, extra)?,
            refresh_token: self.mint_refresh(&user.tenant_id, user.role).await?,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_ttl_secs,
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
}
