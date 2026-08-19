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

use ymir::errors::Outcome;

pub(crate) mod jwt;
pub(crate) mod service;
pub(crate) mod views;

use crate::services::token_service::views::TokenResponse;
pub use common::auth::claims::Claims;
pub use common::auth::middleware::OauthTokenValidator;

#[async_trait::async_trait]
pub trait TokenServiceTrait: OauthTokenValidator + Send + Sync + 'static {
    async fn issue_token(&self, email: &str, password: &str) -> Outcome<TokenResponse>;
    async fn refresh_token(&self, refresh_jwt: &str) -> Outcome<TokenResponse>;
    async fn revoke_refresh_token(&self, refresh_jwt: &str) -> Outcome<()>;
}
