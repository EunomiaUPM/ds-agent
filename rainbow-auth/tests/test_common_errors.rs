// Tests corresponding to 'rainbow-auth\src\ssi_auth\common\errors'

use anyhow::anyhow;
use axum::{http::StatusCode, response::IntoResponse};
use rainbow_auth::ssi_auth::common::errors::AuthErrors;
use rainbow_auth::ssi_auth::common::errors::CustomToResponse;
use rainbow_common::errors::CommonErrors;
use rainbow_common::errors::{ErrorInfo, ErrorLog};

#[cfg(test)]
mod tests {

    use super::*;

    // Test for 'auth_errors.rs'

    #[test]
    fn test_wallet_new() {
        let url = "https://example.com".to_string();
        let method = "POST".to_string();
        let http_code = 502;
        let cause = Some("Timeout".to_string());

        let error = AuthErrors::wallet_new(url.clone(), method.clone(), http_code, cause.clone());

        // Verify that the error information is correct
        if let AuthErrors::WalletError { info, http_code: code, url: u, method: m, cause: c } =
            error
        {
            assert_eq!(info.message, "Unexpected response from the Wallet");
            assert_eq!(info.error_code, 2100);
            assert_eq!(info.status_code, StatusCode::BAD_GATEWAY);
            assert_eq!(code, http_code);
            assert_eq!(u, url);
            assert_eq!(m, method);
            assert_eq!(c, cause);
        } else {
            panic!("Expected WalletError variant");
        }
    }

    #[test]
    fn test_security_new() {
        let cause = Some("Invalid token".to_string());

        let error = AuthErrors::security_new(cause.clone());

        // Verify that the error information is correct
        if let AuthErrors::SecurityError { info, cause: c } = error {
            assert_eq!(info.message, "Invalid petition");
            assert_eq!(info.error_code, 4400);
            assert_eq!(info.status_code, StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(c, cause);
        } else {
            panic!("Expected SecurityError variant");
        }
    }

    #[test]
    fn test_into_response_wallet_error() {
        let url = "https://example.com".to_string();
        let method = "POST".to_string();
        let http_code = 502;
        let cause = Some("Timeout".to_string());

        let error = AuthErrors::wallet_new(url.clone(), method.clone(), http_code, cause.clone());
        let response = error.into_response();

        // Verify that the status code is as expected
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

        // Verify that the response body contains the error information
        let body = tokio::runtime::Runtime::new().unwrap().block_on(async {
            axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
        });
        let error_info: ErrorInfo = serde_json::from_slice(&body).unwrap();
        assert_eq!(error_info.message, "Unexpected response from the Wallet");
    }

    #[test]
    fn test_into_response_security_error() {
        let cause = Some("Invalid token".to_string());

        let error = AuthErrors::security_new(cause.clone());
        let response = error.into_response();

        // Verify that the status code is as expected
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        // Verify that the response body contains the error information
        let body = tokio::runtime::Runtime::new().unwrap().block_on(async {
            axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
        });
        let error_info: ErrorInfo = serde_json::from_slice(&body).unwrap();
        assert_eq!(error_info.message, "Invalid petition");
    }

    #[test]
    fn test_log_wallet_error() {
        let url = "https://example.com".to_string();
        let method = "POST".to_string();
        let http_code = 502;
        let cause = Some("Timeout".to_string());

        let error = AuthErrors::wallet_new(url.clone(), method.clone(), http_code, cause.clone());

        // Verify that the log message contains the expected information
        let log_message = error.log();
        assert!(log_message.contains("Wallet Error"));
        assert!(log_message.contains(&format!("Http Code: {}", http_code)));
        assert!(log_message.contains(&format!("Method: {}", method)));
        assert!(log_message.contains(&format!("Url: {}", url)));
        assert!(log_message.contains("Cause: Timeout"));
    }

    #[test]
    fn test_log_security_error() {
        let cause = Some("Invalid token".to_string());

        let error = AuthErrors::security_new(cause.clone());

        // Verify that the log message contains the expected information
        let log_message = error.log();
        assert!(log_message.contains("Security Error"));
        assert!(log_message.contains("Cause: Invalid token"));
    }

    // Test for 'error_adapter.rs'

    fn dummy_error_info(status: StatusCode) -> ErrorInfo {
        ErrorInfo {
            message: "Test error".to_string(),
            error_code: 4001,
            status_code: status,
            details: Some("Details about the error".to_string())
        }
    }

    #[test]
    fn test_common_petition_error_response() {
        let error = anyhow!(CommonErrors::PetitionError {
            info: dummy_error_info(StatusCode::BAD_REQUEST),
            http_code: Some(400),
            url: "http://example.com".to_string(),
            method: "GET".to_string(),
            cause: "Invalid request".to_string()
        });
        let response = error.to_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_auth_wallet_error_response() {
        let error = anyhow!(AuthErrors::WalletError {
            info: dummy_error_info(StatusCode::UNAUTHORIZED),
            http_code: 401,
            url: "http://wallet.example.com".to_string(),
            method: "POST".to_string(),
            cause: Some("Wallet not found".to_string())
        });
        let response = error.to_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_auth_security_error_response() {
        let error = anyhow!(AuthErrors::SecurityError {
            info: dummy_error_info(StatusCode::FORBIDDEN),
            cause: Some("Invalid token".to_string())
        });
        let response = error.to_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_unhandled_error_response() {
        let error = anyhow!("Some unexpected error");
        let response = error.to_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
