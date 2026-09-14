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

//! Pagination request configuration and bounds validation.

use serde::{Deserialize, Serialize};
use ymir::errors::{BadFormat, Errors, Outcome};

/// Default page size when the client does not specify limit.
pub const DEFAULT_PAGE_LIMIT: u32 = 20;

/// Hard upper bound on limit to protect data stores from unbounded scans.
pub const MAX_PAGE_LIMIT: u32 = 100;

/// Maximum number of ids accepted in a single batch lookup.
pub const MAX_BATCH_IDS: usize = 100;

/// Pagination parameters specifying batch size and cursor or page position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    #[serde(
        default = "Page::default_limit",
        deserialize_with = "deserialize_u32_from_str_or_int"
    )]
    pub limit: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_u32_from_str_or_int"
    )]
    pub page: Option<u32>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            limit: Self::default_limit(),
            cursor: None,
            page: None,
        }
    }
}

impl Page {
    /// Creates a new pagination request specification.
    pub fn new(limit: u32, cursor: Option<String>) -> Self {
        Self {
            limit,
            cursor,
            page: None,
        }
    }

    /// Creates an indexed pagination request specification.
    pub fn new_indexed(limit: u32, page: Option<u32>) -> Self {
        Self {
            limit,
            cursor: None,
            page,
        }
    }

    /// Provides default limit for deserialization.
    pub fn default_limit() -> u32 {
        DEFAULT_PAGE_LIMIT
    }

    /// Clamps an arbitrary limit into the allowed interval.
    pub fn clamp_limit(limit: u32) -> u32 {
        limit.clamp(1, MAX_PAGE_LIMIT)
    }

    /// Returns a copy with clamped limit bounds.
    pub fn clamped(&self) -> Self {
        Self {
            limit: Self::clamp_limit(self.limit),
            cursor: self.cursor.clone(),
            page: self.page,
        }
    }

    /// Validates pagination bounds.
    pub fn validate(&self) -> Outcome<()> {
        if self.limit == 0 {
            return Err(Errors::format(
                BadFormat::Received,
                "page limit must be strictly positive",
                None,
            ));
        }
        Ok(())
    }

    /// Returns lookahead limit count for windowing queries.
    pub fn fetch_limit(&self) -> u64 {
        (Self::clamp_limit(self.limit) as u64) + 1
    }
}

/// Backwards compatible alias for default limit.
pub fn default_limit() -> u32 {
    Page::default_limit()
}

/// Backwards compatible alias for clamping page limit.
pub fn clamp_page_limit(limit: u32) -> u32 {
    Page::clamp_limit(limit)
}

/// Legacy / offset-based pagination parameters with limit and page index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaginationParams {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_u64_from_str_or_int"
    )]
    pub limit: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_u64_from_str_or_int"
    )]
    pub page: Option<u64>,
}

impl PaginationParams {
    /// Creates a new page-indexed pagination parameter specification.
    pub fn new(limit: Option<u64>, page: Option<u64>) -> Self {
        Self { limit, page }
    }

    /// Converts to standard cursor Page with clamped limit.
    pub fn to_page(&self) -> Page {
        let limit = self
            .limit
            .map(|l| Page::clamp_limit(l as u32))
            .unwrap_or(DEFAULT_PAGE_LIMIT);
        Page {
            limit,
            cursor: None,
            page: self.page.map(|p| p as u32),
        }
    }
}

/// Deserializes u32 from either an integer token or a string representation.
pub fn deserialize_u32_from_str_or_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct U32Visitor;

    impl<'de> serde::de::Visitor<'de> for U32Visitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a u32 integer or string containing a u32")
        }

        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            u32::try_from(v).map_err(serde::de::Error::custom)
        }

        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            u32::try_from(v).map_err(serde::de::Error::custom)
        }

        fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<u32>().map_err(serde::de::Error::custom)
        }
    }

    deserializer.deserialize_any(U32Visitor)
}

/// Deserializes Option<u32> from either an integer token, a string, or None.
pub fn deserialize_opt_u32_from_str_or_int<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptU32Visitor;

    impl<'de> serde::de::Visitor<'de> for OptU32Visitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional u32 integer or string containing a u32")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            deserialize_u32_from_str_or_int(deserializer).map(Some)
        }

        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            u32::try_from(v).map(Some).map_err(serde::de::Error::custom)
        }

        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            u32::try_from(v).map(Some).map_err(serde::de::Error::custom)
        }

        fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.is_empty() {
                Ok(None)
            } else {
                v.parse::<u32>().map(Some).map_err(serde::de::Error::custom)
            }
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(OptU32Visitor)
}

/// Deserializes Option<u64> from either an integer token, a string, or None.
pub fn deserialize_opt_u64_from_str_or_int<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptU64Visitor;

    impl<'de> serde::de::Visitor<'de> for OptU64Visitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional u64 integer or string containing a u64")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            deserializer.deserialize_any(OptU64Visitor)
        }

        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            u64::try_from(v).map(Some).map_err(serde::de::Error::custom)
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.is_empty() {
                Ok(None)
            } else {
                v.parse::<u64>().map(Some).map_err(serde::de::Error::custom)
            }
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(OptU64Visitor)
}

/// Deserializes Option<bool> from a boolean token or string ("true"/"false"/"1"/"0").
pub fn deserialize_opt_bool_from_str_or_bool<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptBoolVisitor;

    impl<'de> serde::de::Visitor<'de> for OptBoolVisitor {
        type Value = Option<bool>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional boolean or string 'true'/'false'")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            match v.to_ascii_lowercase().as_str() {
                "true" | "1" => Ok(Some(true)),
                "false" | "0" => Ok(Some(false)),
                "" => Ok(None),
                _ => Err(serde::de::Error::custom(format!(
                    "invalid boolean string: {}",
                    v
                ))),
            }
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(OptBoolVisitor)
}
