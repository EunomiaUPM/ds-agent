use axum::http::HeaderMap;
use ymir::errors::{BadFormat, Errors, Outcome};

pub(crate) fn bearer(headers: &HeaderMap) -> Outcome<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "missing or malformed Authorization header",
                None,
            )
        })
}
