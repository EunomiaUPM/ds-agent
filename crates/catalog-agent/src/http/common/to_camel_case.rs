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

use convert_case::{Case, Casing};
use serde::{Serialize, Serializer};
use serde_json::Value;

pub struct ToCamelCase<T>(pub T);

impl<T: Serialize> Serialize for ToCamelCase<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = serde_json::to_value(&self.0).map_err(serde::ser::Error::custom)?;
        let converted_value = keys_to_camel(value);
        converted_value.serialize(serializer)
    }
}

fn keys_to_camel(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                let new_key = k.to_case(Case::Camel);
                new_map.insert(new_key, keys_to_camel(v));
            }
            Value::Object(new_map)
        }
        Value::Array(vec) => {
            let new_vec = vec.into_iter().map(keys_to_camel).collect();
            Value::Array(new_vec)
        }
        v => v,
    }
}
