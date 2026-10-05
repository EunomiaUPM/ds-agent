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

//! The identity provider of the agent, as far as it depends on ds-agent.
//!
//! The provider-independent part lives in `ymir`: [`UserInfo`] (the user, its role path and the
//! access rule over the role tree), the [`OauthTokenValidatorTrait`] with its two validators, the
//! [`OauthHttpMiddleware`] and the `UserInfo` axum extractor. What stays here needs this
//! repository's config or transports:
//!
//! | `oauth.provider` | Validator ([`token_validator`]) | Token |
//! |---|---|---|
//! | `keycloak` | decodes the JWT that oauth2-proxy verified and forwarded | required |
//! | `static` (default) | every request is the user in the config; refused in production | ignored |
//! | `built_in` | not operational yet: the agent panics at start | — |
//!
//! ## 1. The rule
//!
//! Roles are paths hanging from the root `/admin` (`/admin/upm/dit`); the second segment is the
//! company. A user reaches (reads and writes) its own records and those created under a role
//! below its own, never those of a user sharing its role; `/admin` reaches everything. See
//! [`UserInfo::reaches`].
//!
//! ## 2. Protecting an HTTP router
//!
//! Put [`OauthHttpMiddleware::run`] as a route layer with the validator as state. It stores the
//! user in the request; handlers take it as an extractor.
//!
//! ```rust,ignore
//! pub(crate) fn router(self) -> Router {
//!     Router::new()
//!         .route("/", get(Self::handle_list))
//!         .with_state(self.clone())
//!         .route_layer(from_fn_with_state(self.validator, OauthHttpMiddleware::run))
//! }
//!
//! async fn handle_list(State(s): State<Self>, user: UserInfo) -> AppResult<Json<Page>> {
//!     Ok(Json(s.service.list(&user).await?))
//! }
//! ```
//!
//! ## 3. Authorizing in the service layer
//!
//! Records are stamped with the user's id and role when created. Lists filter in the database
//! by "own or below the caller's role"; single reads and writes check
//! [`UserInfo::ensure_reaches`], which hides unreachable records as a 404.
//!
//! ## 4. gRPC
//!
//! [`GrpcAuth`] does the same from tonic metadata, returning a `Status` on failure.
//!
//! [`GrpcAuth`]: grpc::GrpcAuth
//!
//! ## 5. In-process calls
//!
//! A local facade or a seeder has no request behind it: it acts as [`UserInfo::system`], the
//! root.

pub mod grpc;
pub mod provider;
// Atomic checks on the claims of the former built-in tokens; unused since identity comes from
// `UserInfo`. Kept as it was, out of the module tree.
// pub mod rules;

pub use provider::token_validator;
// pub use rules::AuthRules;
pub use ymir::http::OauthHttpMiddleware;
pub use ymir::services::token_validator::{
    FixedUserValidator, ProxiedTokenValidator, OauthTokenValidatorTrait,
};
pub use ymir::types::oauth::{RolePath, RoleTrait, UserInfo, UserTrait};

/// Header / metadata key carrying the bearer token.
pub const AUTHORIZATION_HEADER: &str = "authorization";

#[cfg(test)]
mod middleware_tests;
#[cfg(test)]
mod tests;
