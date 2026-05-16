use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub(super) fn deser_enum<T: for<'de> Deserialize<'de>>(s: &str) -> Outcome<T> {
    serde_json::from_str(&format!("\"{}\"", s))
        .map_err(|e| Errors::crazy("invalid enum value in database", Some(Box::new(e))))
}

pub(super) fn ser_enum<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .expect("enum serialization never fails")
        .as_str()
        .expect("enum serializes to a string")
        .to_string()
}

pub(super) fn ser_json<T: Serialize>(v: &T) -> Json {
    serde_json::to_value(v).expect("domain type serialization never fails")
}

pub(super) fn deser_json<T: for<'de> Deserialize<'de>>(v: Json, field: &'static str) -> Outcome<T> {
    serde_json::from_value(v).map_err(|e| Errors::crazy(field, Some(Box::new(e))))
}

pub(super) fn parse_urn(s: &str, field: &'static str) -> Outcome<Urn> {
    Urn::from_str(s).map_err(|e| Errors::crazy(field, Some(Box::new(e))))
}
