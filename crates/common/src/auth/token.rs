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

//! Transport-agnostic token verification port for authenticating credentials into claims.

use ymir::errors::Outcome;

use crate::auth::claims::Claims;

/// Port implemented by services capable of verifying bearer tokens and resolving claims.
#[async_trait::async_trait]
pub trait OauthTokenValidator: Send + Sync + 'static {
    async fn validate_token(&self, token: &str) -> Outcome<Claims>;
}

/// Semantic alias for token verification service contracts.
pub use OauthTokenValidator as TokenVerifier;
