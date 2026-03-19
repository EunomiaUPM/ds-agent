/*
 *
 *  * Copyright (C) 2024 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

pub mod error_adapter;
mod error_log_trait;
pub mod helpers;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
pub use error_log_trait::ErrorLog;
use helpers::{BadFormat, MissingAction};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorInfo {
    pub message: String,
    pub error_code: u16,
    #[serde(skip)]
    pub status_code: StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub cause: String
}

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum CommonErrors {
    #[error("Petition Error")]
    PetitionError {
        #[serde(flatten)]
        info: ErrorInfo,
        http_code: Option<u16>,
        url: String,
        method: String,
        cause: String
    },
    #[error("Provider Error")]
    ProviderError {
        #[serde(flatten)]
        info: ErrorInfo,
        http_code: Option<u16>,
        url: String,
        method: String,
        cause: String
    },
    #[error("Consumer Error")]
    ConsumerError {
        #[serde(flatten)]
        info: ErrorInfo,
        http_code: Option<u16>,
        url: String,
        method: String,
        cause: String
    },
    #[error("Authority Error")]
    AuthorityError {
        #[serde(flatten)]
        info: ErrorInfo,
        http_code: Option<u16>,
        url: String,
        method: String,
        cause: String
    },
    #[error("Missing Action Error")]
    MissingActionError {
        #[serde(flatten)]
        info: ErrorInfo,
        action: MissingAction,
        cause: String
    },
    #[error("Missing Resource Error")]
    MissingResourceError {
        #[serde(flatten)]
        info: ErrorInfo,
        resource_id: String,
        cause: String
    },
    #[error("Format Error")]
    FormatError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Unauthorized")]
    UnauthorizedError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Forbidden")]
    ForbiddenError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Database Error")]
    DatabaseError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Feature Not Implemented Error")]
    FeatureNotImplError {
        #[serde(flatten)]
        info: ErrorInfo,
        feature: String,
        cause: String
    },
    #[error("File Read Error")]
    ReadError {
        #[serde(flatten)]
        info: ErrorInfo,
        path: String,
        cause: String
    },
    #[error("File Write Error")]
    WriteError {
        #[serde(flatten)]
        info: ErrorInfo,
        path: String,
        cause: String
    },
    #[error("Parse Error")]
    ParseError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Module not active Error")]
    ModuleNotActiveError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Environment Variable Error")]
    EnvVarError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    },
    #[error("Vault Error")]
    VaultError {
        #[serde(flatten)]
        info: ErrorInfo,
        cause: String
    }
}

impl IntoResponse for &CommonErrors {
    fn into_response(self) -> Response {
        let info = match self {
            CommonErrors::PetitionError { info, .. }
            | CommonErrors::ProviderError { info, .. }
            | CommonErrors::ConsumerError { info, .. }
            | CommonErrors::AuthorityError { info, .. }
            | CommonErrors::MissingActionError { info, .. }
            | CommonErrors::MissingResourceError { info, .. }
            | CommonErrors::FormatError { info, .. }
            | CommonErrors::UnauthorizedError { info, .. }
            | CommonErrors::ForbiddenError { info, .. }
            | CommonErrors::DatabaseError { info, .. }
            | CommonErrors::ReadError { info, .. }
            | CommonErrors::WriteError { info, .. }
            | CommonErrors::ParseError { info, .. }
            | CommonErrors::ModuleNotActiveError { info, .. }
            | CommonErrors::VaultError { info, .. }
            | CommonErrors::EnvVarError { info, .. }
            | CommonErrors::FeatureNotImplError { info, .. } => info
        };

        (info.status_code, Json(info)).into_response()
    }
}

impl ErrorLog for CommonErrors {
    fn log(&self) -> String {
        fn format_info(info: &ErrorInfo, cause: &str) -> String {
            let details = info.details.as_deref().unwrap_or("No details");
            format!(
                "Error Code: {}\nMessage: {}\nDetails: {}\nCause: {}",
                info.error_code, info.message, details, cause
            )
        }

        fn format_http_error(
            info: &ErrorInfo,
            url: &str,
            method: &str,
            http_code: Option<u16>,
            cause: &str
        ) -> String {
            let base = format_info(info, cause);
            let code = http_code.unwrap_or(0);
            format!("{}\nMethod: {}\nUrl: {}\nHttp Code: {}", base, method, url, code)
        }

        match self {
            CommonErrors::PetitionError { info, http_code, url, method, cause }
            | CommonErrors::ProviderError { info, http_code, url, method, cause }
            | CommonErrors::AuthorityError { info, http_code, url, method, cause }
            | CommonErrors::ConsumerError { info, http_code, url, method, cause } => {
                format_http_error(info, url, method, *http_code, cause)
            }
            CommonErrors::MissingActionError { info, action, cause } => {
                format!("{}\nMissingAction: {}", format_info(info, cause), action)
            }
            CommonErrors::FeatureNotImplError { info, feature, cause } => {
                format!("{}\nFeature: {}", format_info(info, cause), feature)
            }
            CommonErrors::MissingResourceError { info, resource_id, cause } => {
                format!("{}\nResource Id: {}", format_info(info, cause), resource_id)
            }
            CommonErrors::ReadError { info, path, cause } => {
                format!("{}\nPath: {}", format_info(info, cause), path)
            }
            CommonErrors::WriteError { info, path, cause } => {
                format!("{}\nPath: {}", format_info(info, cause), path)
            }
            CommonErrors::FormatError { info, cause }
            | CommonErrors::UnauthorizedError { info, cause }
            | CommonErrors::ModuleNotActiveError { info, cause }
            | CommonErrors::ForbiddenError { info, cause }
            | CommonErrors::DatabaseError { info, cause }
            | CommonErrors::EnvVarError { info, cause }
            | CommonErrors::VaultError { info, cause }
            | CommonErrors::ParseError { info, cause } => format_info(info, cause)
        }
    }
}

impl CommonErrors {
    pub fn petition_new(
        url: &str,
        method: &str,
        http_code: Option<u16>,
        cause: &str
    ) -> CommonErrors {
        CommonErrors::PetitionError {
            info: ErrorInfo {
                message: "A petition went wrong".to_string(),
                error_code: 1000,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
                cause: cause.to_string()
            },
            http_code,
            url: url.to_string(),
            method: method.to_string(),
            cause: cause.to_string()
        }
    }
    pub fn provider_new(
        url: &str,
        method: &str,
        http_code: Option<u16>,
        cause: &str
    ) -> CommonErrors {
        CommonErrors::ProviderError {
            info: ErrorInfo {
                message: "Unexpected response from the Provider".to_string(),
                error_code: 2200,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
                cause: cause.to_string()
            },
            http_code,
            url: url.to_string(),
            method: method.to_string(),
            cause: cause.to_string()
        }
    }
    pub fn consumer_new(
        url: &str,
        method: &str,
        http_code: Option<u16>,
        cause: &str
    ) -> CommonErrors {
        CommonErrors::ConsumerError {
            info: ErrorInfo {
                message: "Unexpected response from the Consumer".to_string(),
                error_code: 2300,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
                cause: cause.to_string()
            },
            http_code,
            url: url.to_string(),
            method: method.to_string(),
            cause: cause.to_string()
        }
    }
    pub fn authority_new(
        url: &str,
        method: &str,
        http_code: Option<u16>,
        cause: &str
    ) -> CommonErrors {
        CommonErrors::AuthorityError {
            info: ErrorInfo {
                message: "Unexpected response from the Authority".to_string(),
                error_code: 2400,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
                cause: cause.to_string()
            },
            http_code,
            url: url.to_string(),
            method: method.to_string(),
            cause: cause.to_string()
        }
    }
    pub fn missing_action_new(action: MissingAction, cause: &str) -> CommonErrors {
        let error_code = match action {
            MissingAction::Token => 3110,
            MissingAction::Wallet => 3120,
            MissingAction::Key => 3130,
            MissingAction::Did => 3140,
            MissingAction::Onboarding => 3150,
            _ => 3100
        };
        CommonErrors::MissingActionError {
            info: ErrorInfo {
                message: format!("The action {} is required to proceed with this step", action),
                error_code,
                status_code: StatusCode::PRECONDITION_FAILED,
                details: None,
                cause: cause.to_string()
            },
            action,
            cause: cause.to_string()
        }
    }
    pub fn missing_resource_new(resource_id: &str, cause: &str) -> CommonErrors {
        CommonErrors::MissingResourceError {
            info: ErrorInfo {
                message: "A key resource is missing in order to complete the required action "
                    .to_string(),
                error_code: 3200,
                status_code: StatusCode::NOT_FOUND,
                details: None,
                cause: cause.to_string()
            },
            resource_id: resource_id.to_string(),
            cause: cause.to_string()
        }
    }
    pub fn format_new(option: BadFormat, cause: &str) -> CommonErrors {
        let (error_code, status_code) = match option {
            BadFormat::Sent => (3110, StatusCode::BAD_GATEWAY),
            BadFormat::Received => (3120, StatusCode::BAD_REQUEST),
            _ => (3100, StatusCode::BAD_REQUEST)
        };
        CommonErrors::FormatError {
            info: ErrorInfo {
                message: "Invalid Format".to_string(),
                error_code,
                status_code,
                details: Some(cause.to_string()),
                cause: cause.to_string()
            },
            cause: cause.to_string()
        }
    }
    pub fn unauthorized_new(cause: &str) -> CommonErrors {
        CommonErrors::UnauthorizedError {
            info: ErrorInfo {
                message: "Unauthorized".to_string(),
                error_code: 4200,
                status_code: StatusCode::UNAUTHORIZED,
                details: None,
                cause: cause.to_string()
            },
            cause: cause.to_string()
        }
    }
    pub fn forbidden_new(cause: &str) -> CommonErrors {
        CommonErrors::ForbiddenError {
            info: ErrorInfo {
                message: "Forbidden".to_string(),
                error_code: 4300,
                status_code: StatusCode::FORBIDDEN,
                details: None,
                cause: cause.to_string()
            },
            cause: cause.to_string()
        }
    }
    pub fn database_new(cause: &str) -> CommonErrors {
        CommonErrors::DatabaseError {
            info: ErrorInfo {
                message: "Error related to the database".to_string(),
                error_code: 5100,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
                cause: cause.to_string()
            },
            cause: cause.to_string()
        }
    }
    pub fn not_impl_new(feature: &str, cause: &str) -> CommonErrors {
        CommonErrors::FeatureNotImplError {
            info: ErrorInfo {
                message: "Feature not implemented yet".to_string(),
                error_code: 5200,
                status_code: StatusCode::NOT_IMPLEMENTED,
                details: None,
                cause: cause.to_string()
            },
            feature: feature.to_string(),
            cause: cause.to_string()
        }
    }
    pub fn read_new(path: &str, cause: &str) -> Self {
        Self::ReadError {
            info: ErrorInfo {
                message: format!("Failed to read file {}", path),
                error_code: 6010,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
                cause: cause.to_string()
            },
            path: path.to_string(),
            cause: cause.to_string()
        }
    }

    pub fn write_new(path: &str, cause: &str) -> Self {
        Self::WriteError {
            info: ErrorInfo {
                message: format!("Failed to write file {}", path),
                error_code: 6020,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
                cause: cause.to_string()
            },
            path: path.to_string(),
            cause: cause.to_string()
        }
    }

    pub fn parse_new(cause: &str) -> Self {
        Self::ParseError {
            info: ErrorInfo {
                message: "Failed to parse file".to_string(),
                error_code: 6030,
                status_code: StatusCode::BAD_REQUEST,
                details: None,
                cause: cause.to_string()
            },
            cause: cause.to_string()
        }
    }
    pub fn module_new(module: &str) -> Self {
        Self::ModuleNotActiveError {
            info: ErrorInfo {
                message: "You are trying to use a module which is not active".to_string(),
                error_code: 5500,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
                cause: module.to_string()
            },
            cause: format!("module {} is not active", module)
        }
    }
    pub fn env_new(e: String) -> Self {
        Self::EnvVarError {
            info: ErrorInfo {
                message: "You are trying to use an undefined env variable".to_string(),
                error_code: 800,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
                cause: e.clone()
            },
            cause: e
        }
    }
    pub fn vault_new(e: String) -> Self {
        Self::VaultError {
            info: ErrorInfo {
                message: "Error related to the vault".to_string(),
                error_code: 800,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
                cause: e.clone()
            },
            cause: e
        }
    }
}
