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

//! Lossless-as-possible conversion between `serde_json::Value` and `google.protobuf.Struct`.

use prost_types::value::Kind;
use prost_types::{ListValue, Struct, Value as ProstValue};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tonic::Status;

use crate::grpc::field::InvalidField;

/// Conversions between typed Rust values and `google.protobuf.Struct`.
pub struct JsonStruct;

impl JsonStruct {
    /// Serializes a value into a `Struct`; non-object values yield an empty `Struct`.
    pub fn from_typed<T: Serialize>(value: &T) -> Result<Struct, Status> {
        serde_json::to_value(value)
            .map(JsonValue::into_prost_struct)
            .map_err(|e| Status::internal(format!("failed to serialize to Struct: {e}")))
    }

    /// Deserializes a `Struct` into a typed value, failing with `INVALID_ARGUMENT` on `field`.
    pub fn into_typed<T: DeserializeOwned>(s: Struct, field: &str) -> Result<T, Status> {
        serde_json::from_value(s.into_json()).map_err(|e| InvalidField::status(field, e))
    }

    /// Whole `f64` values become JSON integers so typed integer fields deserialize back.
    fn number(n: f64) -> JsonValue {
        if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
            return JsonValue::from(n as i64);
        }
        serde_json::Number::from_f64(n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)
    }
}

/// `prost_types::{Struct, Value}` → `serde_json::Value`.
pub trait JsonStructExt {
    fn into_json(self) -> JsonValue;
    /// Deserializes into a typed value, failing with `INVALID_ARGUMENT` on `field`.
    fn into_typed<T: DeserializeOwned>(self, field: &str) -> Result<T, Status>
    where
        Self: Sized,
    {
        serde_json::from_value(self.into_json()).map_err(|e| InvalidField::status(field, e))
    }
}

impl JsonStructExt for Struct {
    fn into_json(self) -> JsonValue {
        JsonValue::Object(
            self.fields
                .into_iter()
                .map(|(k, v)| (k, v.into_json()))
                .collect(),
        )
    }
}

impl JsonStructExt for ProstValue {
    fn into_json(self) -> JsonValue {
        match self.kind {
            None | Some(Kind::NullValue(_)) => JsonValue::Null,
            Some(Kind::BoolValue(b)) => JsonValue::Bool(b),
            Some(Kind::NumberValue(n)) => JsonStruct::number(n),
            Some(Kind::StringValue(s)) => JsonValue::String(s),
            Some(Kind::ListValue(l)) => {
                JsonValue::Array(l.values.into_iter().map(ProstValue::into_json).collect())
            }
            Some(Kind::StructValue(s)) => s.into_json(),
        }
    }
}

/// `serde_json::Value` → `prost_types::{Struct, Value}`.
pub trait JsonValueExt {
    fn into_prost_value(self) -> ProstValue;
    /// Objects map field by field; any other JSON value yields an empty `Struct`.
    fn into_prost_struct(self) -> Struct;
}

impl JsonValueExt for JsonValue {
    fn into_prost_value(self) -> ProstValue {
        let kind = match self {
            JsonValue::Null => Kind::NullValue(0),
            JsonValue::Bool(b) => Kind::BoolValue(b),
            // google.protobuf.Value only carries f64; large integers lose precision.
            JsonValue::Number(n) => Kind::NumberValue(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => Kind::StringValue(s),
            JsonValue::Array(a) => Kind::ListValue(ListValue {
                values: a.into_iter().map(JsonValue::into_prost_value).collect(),
            }),
            JsonValue::Object(_) => Kind::StructValue(self.into_prost_struct()),
        };
        ProstValue { kind: Some(kind) }
    }

    fn into_prost_struct(self) -> Struct {
        match self {
            JsonValue::Object(map) => Struct {
                fields: map
                    .into_iter()
                    .map(|(k, v)| (k, v.into_prost_value()))
                    .collect(),
            },
            _ => Struct::default(),
        }
    }
}
