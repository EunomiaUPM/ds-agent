use crate::entities::common::secret_management::SecretString;
use crate::entities::parameters::TemplateString;
use serde::{Deserialize, Serialize};

/// Credentials for HTTP Basic Authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub username: TemplateString,
    pub password: SecretString,
}
