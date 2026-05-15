use std::sync::Arc;

use crate::data::factory::OAuthDataFactory;
use crate::data::in_memory::repos::{InMemoryRefreshTokenRepository, InMemoryUserRepository};
use crate::data::repositories::refresh_token::RefreshTokenRepository;
use crate::data::repositories::user::UserRepository;

pub(crate) struct InMemoryDataFactory {
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
}

impl InMemoryDataFactory {
    pub fn new() -> Self {
        Self {
            user_repo: Arc::new(InMemoryUserRepository::new()),
            refresh_token_repo: Arc::new(InMemoryRefreshTokenRepository::new()),
        }
    }
}

impl OAuthDataFactory for InMemoryDataFactory {
    fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repo.clone()
    }

    fn refresh_token_repository(&self) -> Arc<dyn RefreshTokenRepository> {
        self.refresh_token_repo.clone()
    }
}
