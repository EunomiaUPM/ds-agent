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
