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

#[derive(Clone)]
pub struct SecretValue(serde_json::Value);

impl SecretValue {
    pub fn new(v: serde_json::Value) -> Self {
        Self(v)
    }
    pub fn expose(&self) -> &serde_json::Value {
        &self.0
    }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretValue(*****)")
    }
}

impl serde::Serialize for SecretValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("*****")
    }
}

impl<'de> serde::Deserialize<'de> for SecretValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(SecretValue::new(serde_json::Value::deserialize(
            deserializer,
        )?))
    }
}
