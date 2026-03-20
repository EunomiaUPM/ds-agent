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
    /// # Not yet implemented
    ///
    /// Full resolution (Vault lookup, base64 decode, env-var read) is pending.
    /// For now this always returns an error.  Callers that genuinely need the
    /// resolved value must wait for the implementation to be completed.
    pub async fn resolve(&self) -> Outcome<String> {
        Err(Errors::parse("SecretString::resolve is not yet implemented", None))
    }
}
