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

//! The token validator of the configured identity provider.

use std::sync::Arc;

use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::{Errors, Outcome};
use ymir::services::token_validator::{
    FixedUserValidator, ProxiedTokenValidator, OauthTokenValidatorTrait,
};

use crate::config::OauthConfig;
use crate::config::services::CommonConfig;

/// The validator for `common.oauth`. Fails when `static` is configured in production: it would
/// let anyone in as its fixed user.
///
/// # Panics
/// With `built_in`, which is not operational yet.
pub fn token_validator(common: &CommonConfig) -> Outcome<Arc<dyn OauthTokenValidatorTrait>> {
    match &common.oauth {
        OauthConfig::BuiltIn => {
            panic!("the built_in OAuth provider is not operational yet; use keycloak or static")
        }
        OauthConfig::Keycloak => Ok(Arc::new(ProxiedTokenValidator)),
        OauthConfig::Static(_) if common.connection().is_prod => Err(Errors::security(
            "the static identity provider cannot run in production",
            None,
        )),
        OauthConfig::Static(user) => Ok(Arc::new(FixedUserValidator::new(user.clone()))),
    }
}
