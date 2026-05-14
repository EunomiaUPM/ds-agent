use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use ymir::errors::{BadFormat, Errors};

use crate::entities::ids::{CorrelationId, RequestId, TenantId};

pub(crate) struct ExtractedHeaders {
    pub tenant_id: TenantId,
    /// Echoed from X-Request-ID or generated if absent.
    pub request_id: RequestId,
    pub correlation_id: Option<CorrelationId>,
}

impl ExtractedHeaders {
    /// Base response headers: echoes X-Request-ID and X-Correlation-ID.
    pub fn response_headers(&self) -> HeaderMap {
        let mut map = HeaderMap::new();
        insert_str(&mut map, "x-request-id", self.request_id.as_str());
        if let Some(c) = &self.correlation_id {
            insert_str(&mut map, "x-correlation-id", c.as_str());
        }
        map
    }

    /// Adds X-Total-Count on top of the base response headers (for paginated responses).
    pub fn response_headers_paged(&self, total: Option<u64>) -> HeaderMap {
        let mut map = self.response_headers();
        if let Some(n) = total {
            insert_str(&mut map, "x-total-count", &n.to_string());
        }
        map
    }
}

impl<S: Send + Sync> FromRequestParts<S> for ExtractedHeaders {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tenant_id = get_header_str(&parts.headers, "x-tenant-id")
            .map(TenantId::new)
            .ok_or_else(|| {
                Errors::format(BadFormat::Received, "missing required header: X-Tenant-ID", None)
            })?;

        let request_id = get_header_str(&parts.headers, "x-request-id")
            .map(RequestId::new)
            .unwrap_or_else(RequestId::generate);

        let correlation_id = get_header_str(&parts.headers, "x-correlation-id")
            .map(CorrelationId::new);

        Ok(Self { tenant_id, request_id, correlation_id })
    }
}

fn get_header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let key = HeaderName::from_bytes(name.as_bytes()).ok()?;
    headers.get(&key).and_then(|v| v.to_str().ok())
}

fn insert_str(map: &mut HeaderMap, name: &'static str, value: &str) {
    if let Ok(v) = HeaderValue::from_str(value) {
        map.insert(HeaderName::from_static(name), v);
    }
}