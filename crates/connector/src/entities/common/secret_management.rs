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

//! Secret value storage and resolution.
//!
//! A [`SecretString`] wraps a [`SecretSource`] that describes how a credential
//! value is supplied (`Plain`, `Base64`, or `EnvVar`). All variants accept
//! `{{__PARAM__}}` instance placeholders as well as `{{__RUNTIME_SECRET_{/key}__}}`
//! references resolved at runtime through the `keystore` service (which backs
//! secrets with HashiCorp Vault automatically when Vault is enabled).

use crate::entities::parameters::TemplateString;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use ymir::errors::{Errors, Outcome};

/// Describes where a secret credential is stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecretSource {
    /// Inline value or `{{__RUNTIME_SECRET_{/key}__}}` keystore reference.
    Plain(TemplateString),
    /// Base64-encoded value stored inline.
    Base64(TemplateString),
    /// The name of an environment variable that holds the secret.
    EnvVar(TemplateString),
}

/// An opaque credential value.
///
/// The underlying [`SecretSource`] determines how the value is fetched at
/// runtime via [`resolve`](SecretString::resolve).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretString {
    #[serde(flatten)]
    pub source: SecretSource,
}

impl SecretString {
    /// Resolve the secret to a plaintext string.
    ///
    /// Supported sources: `Plain`, `Base64`, `EnvVar`.
    pub async fn resolve(&self) -> Outcome<String> {
        match &self.source {
            SecretSource::Plain(value) => Ok(value.clone()),
            SecretSource::Base64(encoded) => {
                let bytes = STANDARD.decode(encoded).map_err(|e| {
                    Errors::parse("Failed to decode Base64 secret", Some(Box::new(e)))
                })?;
                String::from_utf8(bytes).map_err(|e| {
                    Errors::parse("Base64 secret is not valid UTF-8", Some(Box::new(e)))
                })
            }
            SecretSource::EnvVar(name) => std::env::var(name).map_err(|_| {
                Errors::parse(format!("Environment variable '{name}' not found"), None)
            }),
        }
    }
}
