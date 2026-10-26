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

//! Plain serde records that travel as JSON-LD: their `@context` and `@type` are written on output
//! and checked on input, without reading the body through RDF.

use serde::de::{DeserializeOwned, Error as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// The JSON-LD context and class a record travels with.
pub trait JsonLdType {
    const CONTEXT: &'static str;
    const TYPE: &'static str;
}

/// `T` as a JSON-LD document. A body without `@context` or `@type` is read as `T`'s; another
/// context or type is refused.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonLd<T>(pub T);

impl<T> JsonLd<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T: JsonLdType> JsonLd<T> {
    /// `value` with `T`'s `@context` and `@type`, when it is an object.
    pub fn tag(value: Value) -> Value {
        match value {
            Value::Object(mut object) => {
                object.insert("@context".into(), Value::String(T::CONTEXT.into()));
                object.insert("@type".into(), Value::String(T::TYPE.into()));
                Value::Object(object)
            }
            other => other,
        }
    }

    /// `value` without its `@context` and `@type`, once they are checked to be `T`'s.
    pub fn untag(value: Value) -> Result<Value, String> {
        let Value::Object(mut object) = value else {
            return Err(format!("a {} must be a JSON object", T::TYPE));
        };
        if let Some(context) = object.remove("@context") {
            if !Self::names(&context, T::CONTEXT) {
                return Err(format!("@context must be {}", T::CONTEXT));
            }
        }
        if let Some(kind) = object.remove("@type") {
            if !Self::names(&kind, T::TYPE) {
                return Err(format!("@type must be {}", T::TYPE));
            }
        }
        Ok(Value::Object(object))
    }

    /// Whether `value` is `expected`, alone or in a list.
    fn names(value: &Value, expected: &str) -> bool {
        match value {
            Value::String(s) => s == expected,
            Value::Array(items) => items.iter().any(|item| item.as_str() == Some(expected)),
            _ => false,
        }
    }
}

impl<T: JsonLdType + Serialize> Serialize for JsonLd<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(&self.0).map_err(serde::ser::Error::custom)?;
        Self::tag(value).serialize(serializer)
    }
}

impl<'de, T: JsonLdType + DeserializeOwned> Deserialize<'de> for JsonLd<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Self::untag(Value::deserialize(deserializer)?).map_err(D::Error::custom)?;
        T::deserialize(value).map(JsonLd).map_err(D::Error::custom)
    }
}
