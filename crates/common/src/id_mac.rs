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

//! Declarative macros for URN- and string-backed newtype identifiers.
//!
//! Paths are fully qualified so the macros expand correctly in any crate,
//! independent of the caller's imports. Consuming crates must depend on
//! `urn`, `serde`, and `uuid`. The generated types are `pub(crate)` to the
//! **caller's** crate.

/// Newtype wrapper around a `Urn` with `new`, `as_urn`, and `Display`.
/// The `gen = "prefix"` form also adds `generate()`, minting `urn:prefix:<uuid>`.
#[macro_export]
macro_rules! urn_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) ::urn::Urn);

        #[allow(dead_code)]
        impl $name {
            pub fn new(urn: ::urn::Urn) -> Self {
                Self(urn)
            }
            pub fn as_urn(&self) -> &::urn::Urn {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    ($name:ident, gen = $prefix:literal) => {
        $crate::urn_id!($name);
        #[allow(dead_code)]
        impl $name {
            pub fn generate() -> Self {
                Self($crate::utils::generate_uuid_urn($prefix))
            }
        }
    };
}

/// Newtype wrapper around a string-like `$inner` (`String` / `CompactString`)
/// with `new`, `as_str`, and `Display`. The `gen` form adds `generate()`.
#[macro_export]
macro_rules! str_id {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) $inner);

        #[allow(dead_code)]
        impl $name {
            pub fn new(s: impl Into<$inner>) -> Self {
                Self(s.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    ($name:ident, $inner:ty, gen) => {
        $crate::str_id!($name, $inner);
        #[allow(dead_code)]
        impl $name {
            pub fn generate() -> Self {
                Self(<$inner>::from(::uuid::Uuid::new_v4().to_string()))
            }
        }
    };
}
