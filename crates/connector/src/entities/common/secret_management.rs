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

//! Secret value storage and deferred resolution.
//!
//! A [`SecretString`] wraps a [`SecretSource`] that describes *where* the
//! actual credential lives.  Fields typed as `SecretString` in connector specs
//! (e.g. passwords, API-key values) are intentionally **not** parameterisable
//! via `{{__NAME__}}` placeholders — the credential storage is opaque to the
//! parameter pipeline.
//!
//! # Resolution (future work)
//!
//! [`SecretString::resolve`] is a placeholder for vault / base64 / env-var
//! resolution.  It is not yet implemented.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use ymir::errors::{Errors, Outcome};

/// Describes where a secret credential is stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecretSource {
    /// Inline plaintext value (for development / testing only).
    Plain(String),
    /// Base64-encoded value stored inline.
    Base64(String),
    /// A reference to a HashiCorp Vault path + key.
    VaultRef { path: String, key: String },
    /// The name of an environment variable that holds the secret.
    EnvVar(String),
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
    /// `VaultRef` resolution requires a vault client and is not yet implemented.
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
                Errors::parse(&format!("Environment variable '{}' not found", name), None)
            }),
            SecretSource::VaultRef { .. } => Err(Errors::parse(
                "VaultRef resolution requires a vault client and is not yet implemented",
                None,
            )),
        }
    }
}
