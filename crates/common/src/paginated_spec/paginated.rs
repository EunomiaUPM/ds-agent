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

//! Standard paginated envelope response model.

use serde::{Deserialize, Serialize};

/// Pagination response with items, cursor, and optional total count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl<T> Paginated<T> {
    /// Creates a new paginated response.
    pub fn new(items: Vec<T>, next_cursor: Option<String>, total: Option<u64>) -> Self {
        Self {
            items,
            next_cursor,
            total,
        }
    }

    /// Creates an empty paginated result.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
            total: Some(0),
        }
    }

    /// Returns the number of items in the current page.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Checks if the current page contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Assembles a paginated response using limit + 1 lookahead windowing.
    pub fn from_window<F>(mut items: Vec<T>, requested_limit: usize, cursor_fn: F) -> Self
    where
        F: FnOnce(&T) -> String,
    {
        let has_more = items.len() > requested_limit;
        if has_more {
            items.truncate(requested_limit);
        }
        let next_cursor = if has_more {
            items.last().map(cursor_fn)
        } else {
            None
        };
        Self {
            items,
            next_cursor,
            total: None,
        }
    }

    /// Assembles a paginated response from items, Page, total count and a cursor extractor.
    pub fn from_page<F>(items: Vec<T>, page: &super::Page, total: Option<u64>, cursor_fn: F) -> Self
    where
        F: FnOnce(&T) -> String,
    {
        let next_cursor = if items.len() == page.limit as usize {
            items.last().map(cursor_fn)
        } else {
            None
        };
        Self {
            items,
            next_cursor,
            total,
        }
    }

    /// Maps paginated items from one type into another.
    pub fn map<U, F>(self, f: F) -> Paginated<U>
    where
        F: FnMut(T) -> U,
    {
        Paginated {
            items: self.items.into_iter().map(f).collect(),
            next_cursor: self.next_cursor,
            total: self.total,
        }
    }
}

impl<T> IntoIterator for Paginated<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Paginated<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
