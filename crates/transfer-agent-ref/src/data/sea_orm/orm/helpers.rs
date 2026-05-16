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
