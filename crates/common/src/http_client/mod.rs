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

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use ymir::errors::{Errors, PetitionFailure};

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Network/Reqwest Error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("HTTP Error {status}: {message}")]
    HttpError {
        status: reqwest::StatusCode,
        message: String,
    },

    #[error("Failed to read response body: {0}")]
    BodyReadError(reqwest::Error),

    #[error("Deserialization Error: {source}")]
    DeserializeError {
        #[source]
        source: serde_json::Error,
        raw_text: String,
    },

    #[error("JSON Serialization Error: {0}")]
    JsonSerializeError(#[from] serde_json::Error),

    #[error("Form Serialization Error: {0}")]
    FormSerializeError(#[from] serde_urlencoded::ser::Error),

    #[error("Semaphore closed")]
    ConcurrencyError,
}

impl From<HttpClientError> for Errors {
    fn from(e: HttpClientError) -> Self {
        match e {
            HttpClientError::RequestError(e) => Errors::petition(
                "unknown",
                "unknown",
                None,
                PetitionFailure::Network,
                "Network error",
                Some(Box::new(e)),
            ),
            HttpClientError::HttpError { status, message } => Errors::petition(
                "unknown",
                "unknown",
                Some(status),
                PetitionFailure::HttpStatus(status),
                message,
                None,
            ),
            HttpClientError::BodyReadError(e) => Errors::petition(
                "unknown",
                "unknown",
                None,
                PetitionFailure::BodyRead,
                "Failed to read body",
                Some(Box::new(e)),
            ),
            HttpClientError::DeserializeError { source, raw_text } => Errors::petition(
                "unknown",
                "unknown",
                None,
                PetitionFailure::BodyDeserialization,
                format!("Raw: {}", raw_text),
                Some(Box::new(source)),
            ),
            HttpClientError::JsonSerializeError(e) => Errors::petition(
                "unknown",
                "unknown",
                None,
                PetitionFailure::Serialization,
                "JSON serialization failed",
                Some(Box::new(e)),
            ),
            HttpClientError::FormSerializeError(e) => Errors::petition(
                "unknown",
                "unknown",
                None,
                PetitionFailure::Serialization,
                "Form serialization failed",
                Some(Box::new(e)),
            ),
            HttpClientError::ConcurrencyError => Errors::petition(
                "unknown",
                "unknown",
                None,
                PetitionFailure::Concurrency,
                "Semaphore closed",
                None,
            ),
        }
    }
}

#[derive(Debug)]
pub struct JsonResponse<T>(pub T);

#[async_trait]
pub trait ApiResponse: Sized {
    async fn from_response(response: reqwest::Response) -> Result<Self, HttpClientError>;
}

#[async_trait]
impl ApiResponse for () {
    async fn from_response(_: reqwest::Response) -> Result<Self, HttpClientError> {
        Ok(())
    }
}

#[async_trait]
impl ApiResponse for String {
    async fn from_response(response: reqwest::Response) -> Result<Self, HttpClientError> {
        response
            .text()
            .await
            .map_err(HttpClientError::BodyReadError)
    }
}

#[async_trait]
impl<T: DeserializeOwned + Send> ApiResponse for JsonResponse<T> {
    async fn from_response(res: reqwest::Response) -> Result<Self, HttpClientError> {
        let json = res
            .json::<T>()
            .await
            .map_err(HttpClientError::BodyReadError)?;
        Ok(JsonResponse(json))
    }
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    client: reqwest::Client,
    auth_token: Arc<RwLock<Option<String>>>,
    limiter: Arc<Semaphore>,
    max_retries: u32,
}

impl HttpClient {
    pub fn new(concurrency_limit: usize, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(concurrency_limit)
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            auth_token: Arc::new(RwLock::new(None)),
            limiter: Arc::new(Semaphore::new(concurrency_limit)),
            max_retries: 1,
        }
    }

    pub async fn set_auth_token(&self, token: String) {
        *self.auth_token.write().await = Some(token);
    }

    pub async fn clear_auth_token(&self) {
        *self.auth_token.write().await = None;
    }

    async fn perform_single_request(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<Bytes>,
        content_type: Option<&str>, // <--- Nuevo parámetro para flexibilidad
    ) -> Result<reqwest::Response, HttpClientError> {
        let mut builder = self.client.request(method, url);
        let token_guard = self.auth_token.read().await;
        if let Some(token) = token_guard.as_ref() {
            builder = builder.bearer_auth(token);
        }
        drop(token_guard);

        if let Some(ct) = content_type {
            builder = builder.header(reqwest::header::CONTENT_TYPE, ct);
        }
        if let Some(b) = body {
            builder = builder.body(b);
        }
        let response = builder.send().await?;

        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            return Err(HttpClientError::HttpError {
                status,
                message: response.text().await.unwrap_or_default(),
            });
        }

        Ok(response)
    }

    async fn execute_with_retries(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<Bytes>,
        content_type: Option<&str>,
    ) -> Result<reqwest::Response, HttpClientError> {
        let mut attempt = 1;

        loop {
            // cheap bytes cloning
            let body_clone = body.clone();

            match self
                .perform_single_request(method.clone(), url, body_clone, content_type)
                .await
            {
                Ok(response) => return Ok(response),
                Err(err) => {
                    if !self.should_retry(&err, attempt) {
                        return Err(err);
                    }
                    let backoff = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                }
            }
        }
    }

    fn should_retry(&self, err: &HttpClientError, attempt: u32) -> bool {
        if attempt > self.max_retries {
            return false;
        }
        match err {
            HttpClientError::RequestError(_) => true,
            HttpClientError::HttpError { status, .. } => {
                status.is_server_error() || *status == reqwest::StatusCode::TOO_MANY_REQUESTS
            }
            _ => false,
        }
    }

    async fn dispatch(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<Bytes>,
        content_type: Option<&str>,
    ) -> Result<reqwest::Response, HttpClientError> {
        let _permit = self
            .limiter
            .acquire()
            .await
            .map_err(|_| HttpClientError::ConcurrencyError)?;
        self.execute_with_retries(method, url, body, content_type)
            .await
    }

    pub async fn get_json<R>(&self, url: &str) -> Result<R, HttpClientError>
    where
        R: DeserializeOwned,
    {
        let response = self.dispatch(reqwest::Method::GET, url, None, None).await?;
        Self::deserialize_internal(response).await
    }

    pub async fn get_json_with_payload<T, R>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<R, HttpClientError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let bytes = serde_json::to_vec(payload)?;
        let body = Bytes::from(bytes);

        let response = self
            .dispatch(
                reqwest::Method::GET,
                url,
                Some(body),
                Some("application/json"),
            )
            .await?;
        Self::deserialize_internal(response).await
    }

    pub async fn post_json<T, R>(&self, url: &str, payload: &T) -> Result<R, HttpClientError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let bytes = serde_json::to_vec(payload)?;
        let body = Bytes::from(bytes);

        let response = self
            .dispatch(
                reqwest::Method::POST,
                url,
                Some(body),
                Some("application/json"),
            )
            .await?;
        Self::deserialize_internal(response).await
    }

    pub async fn post_void<R>(&self, url: &str) -> Result<R, HttpClientError>
    where
        R: ApiResponse,
    {
        let response = self
            .dispatch(reqwest::Method::POST, url, None, None)
            .await?;

        R::from_response(response).await
    }

    pub async fn put_json<T, R>(&self, url: &str, payload: &T) -> Result<R, HttpClientError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let bytes = serde_json::to_vec(payload)?;
        let body = Bytes::from(bytes);
        let response = self
            .dispatch(
                reqwest::Method::PUT,
                url,
                Some(body),
                Some("application/json"),
            )
            .await?;
        Self::deserialize_internal(response).await
    }

    pub async fn delete<R>(&self, url: &str) -> Result<R, HttpClientError>
    where
        R: ApiResponse,
    {
        let response = self
            .dispatch(reqwest::Method::DELETE, url, None, None)
            .await?;
        R::from_response(response).await
    }

    pub async fn post_form<T, R>(&self, url: &str, payload: &T) -> Result<R, HttpClientError>
    where
        T: Serialize,
        R: ApiResponse,
    {
        let form_string = serde_urlencoded::to_string(payload)?;
        let body = Bytes::from(form_string);
        let response = self
            .dispatch(
                reqwest::Method::POST,
                url,
                Some(body),
                Some("application/x-www-form-urlencoded"),
            )
            .await?;
        R::from_response(response).await
    }

    async fn deserialize_internal<R>(response: reqwest::Response) -> Result<R, HttpClientError>
    where
        R: DeserializeOwned,
    {
        let raw_text = response
            .text()
            .await
            .map_err(HttpClientError::BodyReadError)?;
        serde_json::from_str::<R>(&raw_text)
            .map_err(|source| HttpClientError::DeserializeError { source, raw_text })
    }
}
