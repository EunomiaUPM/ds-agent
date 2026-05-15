use std::sync::Arc;

use crate::data::repositories::refresh_token::RefreshTokenRepository;
use crate::data::repositories::user::UserRepository;

pub(crate) trait OAuthDataFactory: Send + Sync {
    fn user_repository(&self) -> Arc<dyn UserRepository>;
    fn refresh_token_repository(&self) -> Arc<dyn RefreshTokenRepository>;
}
