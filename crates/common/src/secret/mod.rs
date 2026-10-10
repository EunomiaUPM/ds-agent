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

//! Secret values that never print or serialize in clear.

/// Secret value; `Debug` and `Serialize` print `*****`, only `expose` gives the content.
#[derive(Clone)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// What `Debug` and `Serialize` print instead of the value.
    pub const MASK: &'static str = "*****";

    pub fn new(v: T) -> Self {
        Self(v)
    }

    /// The value in clear; keep it out of logs and responses.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Takes the value in clear, for persistence.
    pub fn into_exposed(self) -> T {
        self.0
    }
}

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret({})", Self::MASK)
    }
}

impl<T> serde::Serialize for Secret<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(Self::MASK)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Secret<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests;
