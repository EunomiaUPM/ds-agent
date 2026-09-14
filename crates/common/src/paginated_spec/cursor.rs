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

//! URL-safe base64 opaque cursor encoding and decoding.

use base64::Engine;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use ymir::errors::{BadFormat, Errors, Outcome};

/// Decoded cursor components representing pagination state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedCursor {
    pub timestamp: DateTime<FixedOffset>,
    pub id: Option<String>,
}

impl DecodedCursor {
    /// Converts the cursor timestamp into UTC.
    pub fn timestamp_utc(&self) -> DateTime<Utc> {
        self.timestamp.with_timezone(&Utc)
    }
}

/// Utility for opaque cursor serialization and deserialization.
pub struct Cursor;

impl Cursor {
    /// Encodes a timestamp into a URL-safe cursor.
    pub fn encode_timestamp<Tz: TimeZone>(dt: &DateTime<Tz>) -> String
    where
        Tz::Offset: std::fmt::Display,
    {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(dt.to_rfc3339())
    }

    /// Encodes a composite (timestamp, id) into a URL-safe cursor for tie-breaking.
    pub fn encode_composite<Tz: TimeZone>(dt: &DateTime<Tz>, id: &str) -> String
    where
        Tz::Offset: std::fmt::Display,
    {
        let payload = format!("{}#{}", dt.to_rfc3339(), id);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload)
    }

    /// Encodes a timestamp and an optional tie-breaking id into a URL-safe cursor.
    pub fn encode<Tz: TimeZone>(dt: &DateTime<Tz>, id: Option<&str>) -> String
    where
        Tz::Offset: std::fmt::Display,
    {
        match id {
            Some(id) => Self::encode_composite(dt, id),
            None => Self::encode_timestamp(dt),
        }
    }

    /// Decodes a URL-safe cursor into a `DecodedCursor`.
    pub fn decode(cursor: &str) -> Outcome<DecodedCursor> {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    "invalid cursor base64",
                    Some(Box::new(e)),
                )
            })?;
        let s = String::from_utf8(bytes).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "invalid cursor utf8",
                Some(Box::new(e)),
            )
        })?;

        if let Some((ts_str, id_str)) = s.split_once('#') {
            let dt = DateTime::parse_from_rfc3339(ts_str).map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    "invalid cursor rfc3339",
                    Some(Box::new(e)),
                )
            })?;
            Ok(DecodedCursor {
                timestamp: dt,
                id: Some(id_str.to_string()),
            })
        } else {
            let dt = DateTime::parse_from_rfc3339(&s).map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    "invalid cursor rfc3339",
                    Some(Box::new(e)),
                )
            })?;
            Ok(DecodedCursor {
                timestamp: dt,
                id: None,
            })
        }
    }

    /// Decodes a URL-safe cursor into a timestamp with timezone offset.
    pub fn decode_timestamp(cursor: &str) -> Outcome<DateTime<FixedOffset>> {
        Self::decode(cursor).map(|c| c.timestamp)
    }

    /// Decodes a URL-safe cursor into a UTC timestamp.
    pub fn decode_utc_timestamp(cursor: &str) -> Outcome<DateTime<Utc>> {
        Self::decode(cursor).map(|c| c.timestamp_utc())
    }
}
