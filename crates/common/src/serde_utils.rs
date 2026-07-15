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

use base64::Engine;
use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serializer};
use ymir::errors::{Errors, Outcome};

/// Serde custom deserializer for getting b64
pub fn deserialize_b64(s: &str, field: &'static str) -> Outcome<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| Errors::parse(e.to_string(), None))
}

/// Serde custom serializer for serializing json data into base64
pub fn serialize_bytes_b64<S: Serializer>(b: &Bytes, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(b))
}

/// Serde custom serializer for serializing json data into base64 with Option<T> fields
pub fn serialize_opt_bytes_b64<S: Serializer>(b: &Option<Bytes>, s: S) -> Result<S::Ok, S::Error> {
    match b {
        Some(v) => s.serialize_some(&base64::engine::general_purpose::STANDARD.encode(v)),
        None => s.serialize_none(),
    }
}

/// Bytes to hexadecimal conversion util
pub fn bytes_to_hex(h: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(64);
    for b in h {
        write!(buf, "{b:02x}").unwrap();
    }
    buf
}

/// Serde custom serializer for serializing json data into base64 and hex
pub fn serialize_hash_hex<S: Serializer>(h: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&bytes_to_hex(h))
}

/// Serde custom serializer for serializing json data into base64 with hex with Option<T> fields
pub fn serialize_opt_hash_hex<S: Serializer>(
    h: &Option<[u8; 32]>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match h {
        Some(v) => s.serialize_some(&bytes_to_hex(v)),
        None => s.serialize_none(),
    }
}

/// Serde custom deserializer for getting b64
pub fn deserialize_b64_bytes<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
    let s = String::deserialize(d)?;
    deserialize_b64(&s, "bytes")
        .map(Bytes::from)
        .map_err(serde::de::Error::custom)
}
